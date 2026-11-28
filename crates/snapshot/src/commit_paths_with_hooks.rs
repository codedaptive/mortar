//! commit_paths_with_hooks: commit_paths with the folder's commit hooks run, as `git commit` would. It replaces
//! commit_paths once cli and land have moved; a later snapshot order removes commit_paths. Its body is
//! commit_paths_through with a hook step that runs Git::commit_hooks.
//! No unit test: it runs git and the hooks. The integration tests in tests/snapshot/commit_paths_cases.rs cover it:
//! - a refusing pre-commit stops it with its output and leaves the index and commits as they were:
//!   hooks_refusing_pre_commit_stops_commit_paths_with_its_output_and_leaves_index_and_commits;
//! - a refusing commit-msg stops it with no commit and the index as it was:
//!   hooks_refusing_commit_msg_stops_commit_paths_without_a_commit;
//! - a commit-msg rewrite is the commit's message: hooks_commit_msg_rewrite_is_commit_paths_commit_message.

use crate::commit_paths_through::commit_paths_through;
use crate::{CommitPathsError, CommitPathsHooksError, CommitPathsOutcome};
use gitio::{CommitHooksOutcome, Git, Oid};
use std::path::{Path, PathBuf};

/// Commits `paths` onto `parent` as commit_paths does, with the same arguments, and runs the pre-commit and
/// commit-msg hooks as `git commit` would, before the commit is written, on the commit's tree and `message`. A tree
/// equal to `parent`'s (Same) runs none. The commit's message is the one commit-msg leaves.
///
/// Errors: HookRefused when a hook refuses: no commit was written and the index was not touched. Every other error
/// is Paths with commit_paths' error, as commit_paths gives it.
pub fn commit_paths_with_hooks(
    git: &Git,
    folder: &Path,
    parent: &Oid,
    paths: &[PathBuf],
    message: &str,
) -> Result<CommitPathsOutcome, CommitPathsHooksError> {
    commit_paths_through(git, folder, parent, paths, message, |git, tree, message| {
        match git.commit_hooks(tree, message).map_err(CommitPathsError::Git)? {
            CommitHooksOutcome::Message(text) => Ok(text),
            CommitHooksOutcome::Refused(refusal) => Err(CommitPathsHooksError::HookRefused(refusal)),
        }
    })
}
