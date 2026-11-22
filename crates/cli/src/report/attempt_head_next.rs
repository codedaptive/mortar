//! attempt_head_next: the next of a verb refused in an attempt folder whose HEAD is unborn. The request's branch
//! is not the attempt's, so repeat_head_refusal's next would send the agent to the wrong branch; status
//! (status/saved_state_report.rs), undo (undo/reverse_and_step.rs) and mr folder remove
//! (folder/run_folder_remove.rs) take this next instead, and it is the one source of its wording.

use snapshot::SnapshotError;

/// For UnbornHead with `attempt_branch` Some (the short try branch, try/<zone>/<request>/<k>): checking out that
/// branch in the attempt's folder and repeating the same command. Every other error, and None, gives None, so the
/// caller keeps its own next.
pub(crate) fn attempt_head_next(error: &SnapshotError, attempt_branch: Option<&str>) -> Option<String> {
    match (error, attempt_branch) {
        (SnapshotError::UnbornHead { .. }, Some(branch)) => {
            Some(format!("check out the branch {branch} in the attempt's folder, then repeat the same command"))
        }
        _ => None,
    }
}
