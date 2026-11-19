//! The worktree that has the integration branch checked out, found from git's worktree list.
//! No sibling test: it runs git. The RITs crates/cli/tests/cli_rit/run_back_out_tests.rs
//! and crates/cli/tests/cli_rit/run_publish_tests.rs cover it through the binary.

use std::path::PathBuf;

use gitio::Git;

/// The first worktree that is neither bare nor prunable whose branch is "refs/heads/<integration_branch>"; None
/// when no worktree has it checked out. It is found from git alone: the integration folder is not a Mortar folder
/// kind, so it has no folder record. A worktree list git cannot give is the cli's reason "the worktrees could not
/// be listed".
pub fn integration_folder(git: &Git, integration_branch: &str) -> Result<Option<PathBuf>, String> {
    let branch = format!("refs/heads/{integration_branch}");
    let worktrees = git.worktree_list().map_err(|_| "the worktrees could not be listed".to_string())?;
    Ok(worktrees
        .into_iter()
        .find(|worktree| !worktree.bare && !worktree.prunable && worktree.branch.as_deref() == Some(branch.as_str()))
        .map(|worktree| worktree.path))
}
