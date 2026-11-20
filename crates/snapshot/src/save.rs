//! save: save_with_snapshot without the snapshot fact. save_with_snapshot.rs has the behaviour, its errors and the
//! integration cases, which run through save. No unit test: it holds no decision.

use crate::save_with_snapshot::save_with_snapshot;
use crate::{SaveOutcome, SnapshotError, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// Saves `folder`'s files as save_with_snapshot does, with the same arguments and errors, and returns only the
/// SaveOutcome. A caller that needs to know whether the snapshot ref moved calls save_with_snapshot.
pub fn save(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SaveOutcome, SnapshotError> {
    save_with_snapshot(git, folder, target, sentence, extra_excludes, size_cap).map(|saved| saved.outcome)
}
