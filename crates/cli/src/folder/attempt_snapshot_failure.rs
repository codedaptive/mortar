//! The report of attempts_saved's snapshot of an attempt folder that failed or was refused. An unborn HEAD there
//! means the try ref vanished between attempts_of's listing and the snapshot (attempts_of gives a folder only to a
//! worktree with the try branch checked out), so the folder's HEAD names a branch that no longer exists. The next is
//! a repeat: mr folder keep and drop run folders' remake_orphans before attempts_saved, and it re-makes the orphan
//! attempt from its trysnap ref (one with no trysnap ref is refused there instead). attempt_snapshot_failure_tests
//! covers it.

use snapshot::SnapshotError;

use crate::join::snapshot_failure;
use crate::report::{head_refusal, Report};

const NEXT: &str = "repeat the same command";

/// An unborn HEAD is head_refusal for `verb`, its lines the error's Display and that `branch` (without refs/heads/)
/// no longer exists. A detached HEAD is head_refusal with the error's Display alone; attempts_saved does not reach
/// it, since snapshot's saved_state returns UnbornHead and never DetachedHead. Each has next "repeat the same
/// command": an unborn HEAD's repeat passes once remake_orphans re-makes the orphan from its trysnap ref; a detached
/// HEAD's would not, as it is no orphan. Every other error is snapshot_failure's report.
pub(crate) fn attempt_snapshot_failure(verb: &str, branch: &str, error: SnapshotError) -> Report {
    match error {
        SnapshotError::UnbornHead { .. } => {
            let short = branch.strip_prefix("refs/heads/").unwrap_or(branch);
            head_refusal(verb, vec![error.to_string(), format!("the branch {short} no longer exists")], NEXT)
        }
        SnapshotError::DetachedHead { .. } => head_refusal(verb, vec![error.to_string()], NEXT),
        _ => snapshot_failure(verb, error),
    }
}
