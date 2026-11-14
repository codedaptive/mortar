//! capped_excludes: the caller's extra excludes plus one literal exclude per refused file.

use crate::literal_exclude::literal_exclude;
use crate::refused_file::RefusedFile;
use crate::snapshot_error::SnapshotError;

/// `extra` in order, then each refused path as a literal exclude, in order. A refused path that is not UTF-8
/// cannot be passed as an exclude, so it fails with PathNotUtf8 naming it.
pub(crate) fn capped_excludes(extra: &[String], refused: &[RefusedFile]) -> Result<Vec<String>, SnapshotError> {
    let mut excludes = extra.to_vec();
    for file in refused {
        let exclude = literal_exclude(&file.path);
        excludes.push(exclude.ok_or_else(|| SnapshotError::PathNotUtf8 { path: file.path.clone() })?);
    }
    Ok(excludes)
}
