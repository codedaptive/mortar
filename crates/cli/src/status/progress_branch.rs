//! progress_branch: the branch whose progress `mr status` reads for a folder: the request branch, or attempt
//! k's own try branch.

use folders::plan::{attempt_branch, request_branch, PlanError};
use folders::FolderKind;

/// The short branch name (no refs/heads/) for a folder of `kind` in `zone` for `request`: attempt k's
/// try/<zone>/<request>/<k> (attempt_branch); a request or zone folder's req/<zone>/<request> (request_branch).
/// A name or attempt number the plan refuses is its PlanError.
pub fn progress_branch(zone: &str, request: &str, kind: FolderKind) -> Result<String, PlanError> {
    match kind {
        FolderKind::Attempt(k) => attempt_branch(zone, request, k),
        FolderKind::Request | FolderKind::Zone => request_branch(zone, request),
    }
}
