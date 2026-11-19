//! run_restore_backup: `mr restore-backup <host>`, a host's backup on the remote brought back into the local
//! store, under the land lock (land's restore_backup). A local ref at another commit than its backup is
//! listed, never moved. --key names the lock holder only, never passed to the op log.
//! No sibling test: it takes a real lock and runs git, the op log and land;
//! parse_restore_backup_args_tests, restore_backup_report_tests and backup_error_report_tests cover its decisions.
//! Repo integration tests to write (cross-zone, not written here): backup then restore-backup round trip
//! into a fresh clone; restore-backup with a local ref at another commit (Differs, left as it is); restore-backup of
//! a host with no backup; the land lock held by a live pid, exit 3; without a config, exit 1.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{backup_error_report, parse_restore_backup_args, restore_backup_report};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "restore-backup";

/// The host named by parse_restore_backup_args (an Err is restore-backup's usage report, exit 4, before any git);
/// then, under the land lock after recover (locked_act), keyed by --key or op_key("restore-backup", the common
/// dir), land's restore_backup of that host from the config's remote as the context's actor, reported by
/// restore_backup_report or, when refused, backup_error_report naming that remote.
pub fn run_restore_backup(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let host = match parse_restore_backup_args(&args.rest) {
        Ok(host) => host,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let Some(config) = &context.config else {
        return failure_report(
            VERB,
            "no config for a verb that needs it; the shell loads it before restore-backup runs",
        );
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let refs = guard_refs_for(config);
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        Ok(match land::restore_backup(&git, &refs, &host, &context.actor) {
            Ok(restored) => restore_backup_report(&host, &restored),
            Err(error) => backup_error_report(VERB, &error, &refs.remote),
        })
    })
    .unwrap_or_else(|report| report)
}
