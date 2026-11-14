//! Runs a verb's act under its oplog lock, after recover (S-16, S-25).
//! Shell API with no caller yet: the ref-moving verbs of WO-cli-2..6 call it.
//! No unit test: it takes a real lock file and runs git and the op log (S-5); lock_report_tests and
//! failure_report_tests cover what it reports. Nagatha's integration group is to cover, with the mortar binary
//! as a child process in a throwaway repository and the first ref-moving verb: a lock held by a live pid is
//! exit 3 naming the holder's pid, host and since, and the act does not run; an open intent left in the op log
//! is settled by recover before the act runs; the lock file is gone after the act, whether it was done or
//! refused; an act's refusal comes back as its own report.

use std::path::Path;

use gitio::Git;
use oplog::{recover, with_lock, HeldLockError, LockName, PidProbe};

use super::lock_report;
use crate::report::{failure_report, Report};

/// Holds `name` under `common` for op key `key`, runs oplog recover, then `act`. A lock refusal is
/// lock_report; recover or the op log failing is failure_report; `act`'s own report comes back as it is.
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
