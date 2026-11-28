//! save_from_snapshot_through: the body of save_from_snapshot and save_from_snapshot_with_hooks, with the hook step
//! as a parameter. It reads the sentence and HEAD (save_message, save_head) and hands the passed snapshot to
//! save_snapshot_onto with `hooks` passed on.
//! No unit test: it runs git; save_from_snapshot.rs lists its decisions and the integration cases.

use crate::save_head::save_head;
use crate::save_message::save_message;
use crate::save_snapshot_onto::save_snapshot_onto;
use crate::{SavedWithSnapshot, SnapshotError, SnapshotOutcome, SnapshotTarget};
use gitio::{Git, Oid};
use std::path::Path;

/// Saves `folder`'s files from `snap` as save_from_snapshot says, with `hooks` as save_snapshot_onto's hook step: it
/// runs on the written shape only, before the commit is written, and answers the message to commit. Errors are
/// save_from_snapshot's, then `hooks`' own.
pub(crate) fn save_from_snapshot_through<E: From<SnapshotError>>(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    built_on: &Oid,
    snap: SnapshotOutcome,
    hooks: impl FnOnce(&Git, &Oid, String, &[(String, String)]) -> Result<String, E>,
) -> Result<SavedWithSnapshot, E> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let subject = save_message(sentence)?;
    let head = save_head(folder, git.head(folder).map_err(SnapshotError::Git)?)?;
    let snap = SnapshotOutcome { written: false, ..snap };
    save_snapshot_onto(&git, target, &subject, head, built_on.clone(), snap, hooks)
}
