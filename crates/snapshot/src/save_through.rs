//! save_through: the body of save_with_snapshot and save_with_hooks, with the hook step as a parameter. It reads the
//! sentence and HEAD (save_message, save_head), takes the snapshot, and hands the rest to save_snapshot_onto with
//! HEAD as the commit the snapshot was built on and `hooks` passed on.
//! No unit test: it runs git and holds no decision of its own; save_with_snapshot.rs lists its decisions and the
//! integration cases, which run through save.

use crate::save_head::save_head;
use crate::save_message::save_message;
use crate::save_snapshot_onto::save_snapshot_onto;
use crate::snapshot::snapshot;
use crate::{SavedWithSnapshot, SnapshotError, SnapshotTarget};
use gitio::{Git, Oid};
use std::path::Path;

/// Saves `folder`'s files as save_with_snapshot says, with `hooks` as save_snapshot_onto's hook step: it runs on the
/// written shape only, before the commit is written, and answers the message to commit. Errors are
/// save_with_snapshot's, then `hooks`' own.
pub(crate) fn save_through<E: From<SnapshotError>>(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    extra_excludes: &[String],
    size_cap: u64,
    hooks: impl FnOnce(&Git, &Oid, String, &[(String, String)]) -> Result<String, E>,
) -> Result<SavedWithSnapshot, E> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let subject = save_message(sentence)?;
    let (branch, head) = save_head(folder, git.head(folder).map_err(SnapshotError::Git)?)?;
    let snap = snapshot(&git, folder, target, extra_excludes, size_cap)?;
    save_snapshot_onto(&git, target, &subject, (branch, head.clone()), head, snap, hooks)
}
