//! run_sweep: `mr sweep [--idle-days <n>] [--ref <full ref name>]...`, the refs folders' sweep picks (closed
//! requests' snapshot refs, joined leftover branches, replay records of zones the config no longer lists, the
//! caller's extra refs) deleted in one op, then, only with --idle-days, the build dirs of
//! folders idle that long cleared. Under the folders lock after recover: sweep_refs deletes refs
//! of every zone, so no one join lock covers it. --key names the lock holder only, never passed to the op log:
//! sweep_refs draws its own op id, and a repeat finds nothing left to sweep.
//! No sibling test: it takes a real lock and runs git, the op log and folders; parse_sweep_args_tests,
//! config_zones_tests, no_zones_report_tests, folders_error_report_tests, sweep_report_tests and
//! sweep_build_failure_report_tests cover its decisions.
//! Validation tests to write (not written here): sweep with an extra ref deletes it in
//! one folder-sweep-refs op with the actor; sweep keeps a configured zone's refs/mortar/replayed/ record and deletes a
//! removed zone's; sweep --idle-days 0 clears build dirs, without it none; a refused extra ref (refs/heads/...)
//! deletes nothing, exit 1; sweep with a config of no zones refused, exit 1, nothing deleted; the folders lock held
//! by a live pid, exit 3; without a config, exit 1.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_sweep_args, sweep_build_failure_report, sweep_report};
use crate::args::{usage_report, UsageError};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;
use crate::upkeep::{config_zones, folders_error_report, no_zones_report};

const VERB: &str = "sweep";

/// The words read by parse_sweep_args (an Err is sweep's usage report, exit 4, before the verb runs git); the config's zones
/// (config_zones; none is no_zones_report, exit 1). Then, under the folders lock after recover (locked_act), keyed
/// by --key or op_key("sweep", the common dir): folders' sweep_refs as the context's actor over every zone, the
/// extra refs and the protected refs (a refusal or failure is folders_error_report, nothing deleted); then, only
/// with --idle-days, sweep_build (a failure is sweep_build_failure_report naming the refs already deleted);
/// reported by sweep_report.
pub fn run_sweep(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let words = match parse_sweep_args(&args.rest, args.dash_dash) {
        Ok(words) => words,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before sweep runs");
    };
    let Some(zones) = config_zones(config) else {
        return no_zones_report(VERB);
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Folders, &key, &KillPidProbe, VERB, || {
        let deleted = folders::sweep_refs(&git, &context.actor, &zones, &words.refs, &context.protected)
            .map_err(|error| folders_error_report(VERB, &error))?;
        let build = match words.idle_days {
            Some(days) => Some(
                folders::sweep_build(&git, days).map_err(|error| sweep_build_failure_report(&deleted, &error))?,
            ),
            None => None,
        };
        Ok(sweep_report(&deleted, build.as_ref()))
    })
    .unwrap_or_else(|report| report)
}
