//! snapshot: save a folder's files as a commit on the request's snapshot ref, with no formatter.
//! No unit test: it only calls snapshot_formatted with no formatter (S-5). Its integration cases are those
//! snapshot_formatted's header lists that need no formatter: a first snapshot, an unchanged folder, chaining,
//! RefMoved, and a 6 MB file refused.

use crate::snapshot_formatted::snapshot_formatted;
use crate::{SnapshotError, SnapshotOutcome, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// snapshot_formatted with no formatter: saves `folder`'s files, minus the built-in excludes, `extra_excludes` and
/// each new file over `size_cap` bytes, on `refs/mortar/snap/<zone>/<request>`, under the same rules and errors.
/// The outcome's formatted is None.
pub fn snapshot(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SnapshotOutcome, SnapshotError> {
    snapshot_formatted(git, folder, target, extra_excludes, size_cap, None)
}
