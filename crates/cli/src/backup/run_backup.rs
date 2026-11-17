//! run_backup: `mortar backup`, this host's in-flight refs pushed to its backup namespace on the remote, under the
//! land lock (SPEC-cli 06: land backup). --key names the lock holder only, never passed to the op log.
//! No sibling test (S-104): it takes a real lock and runs git, the op log and land; backup_report_tests and
//! backup_error_report_tests cover its decisions.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-135, S-136): backup then restore-backup round trip
//! into a fresh clone; a second backup updates nothing; a remote hook refusing refs/mortar-backup/ (Refused); the
//! land lock held by a live pid, exit 3; without a config, exit 1.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{backup_error_report, backup_report};
use crate::args::no_arguments_report;
use crate::guard::guard_refs_for;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "backup";

/// Any argument is backup's usage report, before any git. Then, under the land lock after recover (locked_act),
/// keyed by --key or op_key("backup", the common dir), land's backup as the context's actor to the config's
/// remote, reported by backup_report or, when refused, backup_error_report naming that remote.
pub fn run_backup(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    if let Some(usage) = no_arguments_report(VERB, &args.rest, args.json) {
        return usage;
    }
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before backup runs");
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let refs = guard_refs_for(config);
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        Ok(match land::backup(&git, &refs, &context.actor) {
            Ok(answer) => backup_report(&answer),
            Err(error) => backup_error_report(VERB, &error, &refs.remote),
        })
    })
    .unwrap_or_else(|report| report)
}
