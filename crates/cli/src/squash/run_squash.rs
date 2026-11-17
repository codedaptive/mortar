//! run_squash: `mortar squash <zone>`, a configured zone's changes squashed onto origin as one commit at
//! refs/mortar/squash/<zone> under the land lock (SPEC-cli 06). --key names the lock holder only,
//! never passed to the op log (S-132): repeat safety comes from land's answers (Same).
//! No sibling test (S-104): it takes a real lock and runs git, the op log and land; parse_squash_args_tests,
//! squash_report_tests and landing's squash_error_report_tests cover its decisions. The RIT
//! crates/cli/tests/cli_rit/run_squash_tests.rs covers it through the binary.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119): the land lock held by a live pid,
//! exit 3; without a config, exit 1, next mortar init; the SquashError Git, Oplog, ChangeId, Trailers, OpFailed and
//! OpRepeated texts when git or the op log fails for real; ACCEPT-1 checks 7 and 9.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_squash_args, squash_report};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::landing::squash_error_report;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "squash";

/// The config's zone read by parse_squash_args (an Err is squash's usage report, exit 4, before any git); then,
/// under the land lock after recover (locked_act), keyed by --key or op_key("squash", the common dir), land's
/// squash of that zone onto the config's origin branch as the context's actor, reported by squash_report or, when
/// refused, squash_error_report.
pub fn run_squash(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before squash runs");
    };
    let zone = match parse_squash_args(&args.rest, config) {
        Ok(zone) => zone,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        Ok(match land::squash(&git, &guard_refs_for(config), &zone, &context.actor) {
            Ok(answer) => squash_report(&zone, &answer),
            Err(error) => squash_error_report(VERB, &error),
        })
    })
    .unwrap_or_else(|report| report)
}
