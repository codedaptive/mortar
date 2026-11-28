//! run_save_hooks: the hook step the hooked saves pass to save_snapshot_onto: run the commit hooks on the tree and
//! the built message, and answer the message to commit with the Mortar trailers kept.
//! No unit test: it runs git and the hooks; keeps_save_trailers is its decision and has its tests. The integration
//! cases are listed in save_with_hooks.rs (tests/snapshot/save_cases.rs) and save_from_snapshot_with_hooks.rs
//! (tests/snapshot/save_from_snapshot_cases.rs).

use crate::keeps_save_trailers::keeps_save_trailers;
use crate::{SaveHooksError, SnapshotError};
use gitio::{CommitHooksOutcome, Git, Oid};

/// Runs the pre-commit and commit-msg hooks on `tree` and `message` (Git::commit_hooks) and answers the message
/// commit-msg leaves. A refusing hook is HookRefused; a failing git run is Snapshot(Git).
///
/// A commit-msg hook may drop or change the Mortar trailers `pairs`, so they are added again with
/// Git::with_trailers when the hook's message no longer carries exactly them (keeps_save_trailers). They are not
/// added unconditionally because git's default trailer rule compares a new trailer only with the last one, so a
/// message that still carries them would get all three again.
pub(crate) fn run_save_hooks(
    git: &Git,
    tree: &Oid,
    message: String,
    pairs: &[(String, String)],
) -> Result<String, SaveHooksError> {
    let text = match git.commit_hooks(tree, &message).map_err(SnapshotError::Git)? {
        CommitHooksOutcome::Message(text) => text,
        CommitHooksOutcome::Refused(refusal) => return Err(SaveHooksError::HookRefused(refusal)),
    };
    if keeps_save_trailers(&git.trailers(&text).map_err(SnapshotError::Git)?, pairs) {
        return Ok(text);
    }
    Ok(git.with_trailers(&text, pairs).map_err(SnapshotError::Git)?)
}
