//! Runs a verb's act under its oplog lock, after recover (S-16, S-25).
//! Shell API: `mortar folder` (run_folder's runners), `mortar join` (run_join), `mortar need` (run_need),
//! `mortar resolve` (run_resolve), `mortar restore` (run_restore), `mortar save` (run_save), `mortar guard`
//! (run_guard_install, run_guard_status), `mortar squash` (run_squash), `mortar rereview` (run_rereview),
//! `mortar back-out` (run_back_out), `mortar publish` (run_publish), `mortar would-merge` (run_would_merge),
//! `mortar bisect` (run_bisect), `mortar backup` (run_backup), `mortar restore-backup` (run_restore_backup),
//! `mortar land` (run_land), `mortar after-publish` (run_after_publish), `mortar snapshot` (run_snapshot) and
//! `mortar sweep` (run_sweep) call it, and so will each later verb that moves a ref.
//! No unit test: it takes a real lock file and runs git and the op log (S-5); crates/cli/tests/lock_report_tests.rs and
//! failure_report_tests cover what it reports. Nagatha's integration group is to cover, with the mortar binary as a
//! child process in a throwaway repository and `mortar join`: a lock held by a live pid is exit 3 naming the holder's
//! pid, host and since, and the act does not run; an open intent left in the op log is settled by recover before the
//! act runs; the lock file is gone after the act, whether it was done or refused; an act's refusal comes back as its
//! own report.

use std::path::Path;

use gitio::Git;
use oplog::{recover, with_lock, HeldLockError, LockName, PidProbe};

use super::lock_report;
use crate::report::{failure_report, Report};

/// Holds `name` under `common`, recording `key` as the lock holder's op key (not passed to the op log's apply,
/// S-90), runs oplog recover, then `act`. A lock refusal is lock_report; recover or the op log
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
