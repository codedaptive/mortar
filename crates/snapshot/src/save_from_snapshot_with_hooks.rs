//! save_from_snapshot_with_hooks: save_from_snapshot with the folder's commit hooks run, as `git commit` would. It
//! replaces save_from_snapshot once cli has moved; a later snapshot order removes save_from_snapshot. Its body is
//! save_from_snapshot_through with run_save_hooks as the hook step.
//! No unit test: it runs git and the hooks. The integration test in tests/snapshot/save_from_snapshot_cases.rs covers
//! a refusing pre-commit, which stops the save and leaves the branch:
//! hooks_refusing_pre_commit_stops_save_from_snapshot_and_leaves_the_branch. The commit-msg and trailer cases run
//! through save_with_hooks (save_with_hooks.rs).

use crate::run_save_hooks::run_save_hooks;
use crate::save_from_snapshot_through::save_from_snapshot_through;
use crate::{SaveHooksError, SavedWithSnapshot, SnapshotOutcome, SnapshotTarget};
use gitio::{Git, Oid};
use std::path::Path;

/// Saves `folder`'s files from `snap` as save_from_snapshot does, with the same arguments, and runs the pre-commit
/// and commit-msg hooks as `git commit` would, before the commit is written, on `snap`'s tree and the built message.
/// A save that writes no commit (the unchanged shape) runs none. The committed message is the one commit-msg leaves,
/// with Mortar-Change-Id, Mortar-Request and Mortar-Zone added again when the hook dropped or changed them.
///
/// Errors: HookRefused when a hook refuses: no commit was written, no branch moved, no op was logged and the
/// folder's index is as it was. Every other error is Snapshot with save_from_snapshot's error, as it gives it.
pub fn save_from_snapshot_with_hooks(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    built_on: &Oid,
    snap: SnapshotOutcome,
) -> Result<SavedWithSnapshot, SaveHooksError> {
    save_from_snapshot_through(git, folder, target, sentence, built_on, snap, run_save_hooks)
}
