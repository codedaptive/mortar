//! Points a folder's HEAD at its branch.
//! No unit test: it runs git; attach_head_request_tests covers the run's arguments. The
//! integration group is to cover: a worktree added detached at a branch's commit lists that branch after
//! attach_head, and its HEAD commit is unchanged.

use std::path::Path;

use gitio::Git;

use super::attach_head_request::attach_head_request;
use super::FolderError;

/// Runs attach_head_request(`folder`, `branch`) through `git`.
pub(crate) fn attach_head(git: &Git, folder: &Path, branch: &str) -> Result<(), FolderError> {
    let (args, opts) = attach_head_request(folder, branch);
    git.run(&args, &opts).map(drop).map_err(FolderError::Git)
}
