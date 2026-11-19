//! Runs a verb's act under its oplog lock, after recover.
//! Shell API: `mr folder` (run_folder's runners), `mr join` (run_join), `mr need` (run_need),
//! `mr resolve` (run_resolve), `mr restore` (run_restore), `mr save` (run_save), `mr guard`
//! (run_guard_install, run_guard_status), `mr squash` (run_squash), `mr rereview` (run_rereview),
//! `mr back-out` (run_back_out), `mr publish` (run_publish), `mr would-merge` (run_would_merge),
//! `mr bisect` (run_bisect), `mr backup` (run_backup), `mr restore-backup` (run_restore_backup),
//! `mr land` (run_land), `mr after-publish` (run_after_publish), `mr snapshot` (run_snapshot),
//! `mr sweep` (run_sweep) and `mr undo` (run_undo) call it, and so will each later verb that moves a ref.
//! No unit test: it takes a real lock file and runs git and the op log; lock_report_tests and
//! failure_report_tests cover what it reports. The validation tests are to cover, with the mortar binary as a
//! child process in a throwaway repository and `mr join`: a lock held by a live pid is exit 3 naming the holder's
//! pid, host and since, and the act does not run; an open intent left in the op log is settled by recover before
//! the act runs; the lock file is gone after the act, whether it was done or refused; an act's refusal comes back
//! as its own report.

use std::path::Path;

use gitio::Git;
use oplog::{recover, with_lock, HeldLockError, LockName, PidProbe};

use super::lock_report;
use crate::report::{failure_report, Report};

/// Holds `name` under `common`, recording `key` as the lock holder's op key (not passed to the op log's apply),
/// runs oplog recover, then `act`. A lock refusal is lock_report; recover or the op log
/// failing is failure_report; `act`'s own report comes back as it is.
pub fn locked_act<T>(
    git: &Git,
    common: &Path,
    name: &LockName,
    key: &str,
    pids: &dyn PidProbe,
    verb: &str,
    act: impl FnOnce() -> Result<T, Report>,
) -> Result<T, Report> {
    with_lock(git, common, name, key, pids, || {
        recover(git, common).map_err(|error| failure_report(verb, &error.to_string()))?;
        act()
    })
    .map_err(|error| match error {
        HeldLockError::Lock(error) => lock_report(verb, &error),
        HeldLockError::Oplog(error) => failure_report(verb, &error.to_string()),
        HeldLockError::Work(report) => report,
    })
}
