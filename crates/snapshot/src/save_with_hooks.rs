//! save_with_hooks: save with the folder's commit hooks run, as `git commit` would. It replaces save once cli has
//! moved; a later snapshot order removes save and save_with_snapshot. Its body is save_through with run_save_hooks
//! as the hook step.
//! No unit test: it runs git and the hooks. The integration tests in tests/snapshot/save_cases.rs cover it, in a
//! throwaway repository:
//! - a refusing pre-commit stops the save with its output and leaves the branch, index and commits as they were, but
//!   keeps the snapshot its snapshot step wrote:
//!   hooks_refusing_pre_commit_stops_save_with_its_output_and_leaves_branch_index_and_commits_but_keeps_its_snapshot;
//! - a refusing commit-msg does the same:
//!   hooks_refusing_commit_msg_stops_save_and_leaves_branch_index_and_commits_but_keeps_its_snapshot;
//! - a commit-msg rewrite of the subject is the commit's message:
//!   hooks_commit_msg_rewrite_of_the_subject_is_the_saved_commits_message;
//! - a commit-msg that drops the trailers still gets each once, and the next save reuses the change ID:
//!   hooks_commit_msg_that_drops_the_trailers_still_saves_each_once_and_the_next_save_reuses_the_id.

use crate::run_save_hooks::run_save_hooks;
use crate::save_through::save_through;
use crate::{SaveHooksError, SaveOutcome, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// Saves `folder`'s files as save does, with the same arguments, and runs the pre-commit and commit-msg hooks as
/// `git commit` would, before the commit is written, on the saved tree and the built message. A save that writes no
/// commit (the tree equal to HEAD's) runs none. The committed message is the one commit-msg leaves, with
/// Mortar-Change-Id, Mortar-Request and Mortar-Zone added again when the hook dropped or changed them.
///
/// Errors: HookRefused when a hook refuses: no save commit was written, no branch moved, no save op was logged and
/// the folder's index is as it was; a snapshot step that wrote keeps its snapshot ref and its op. Every other error
/// is Snapshot with save's error, as save gives it.
pub fn save_with_hooks(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SaveOutcome, SaveHooksError> {
    save_through(git, folder, target, sentence, extra_excludes, size_cap, run_save_hooks).map(|saved| saved.outcome)
}
