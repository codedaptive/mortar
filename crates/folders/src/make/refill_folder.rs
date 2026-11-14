//! Fills a folder a repeat found, then returns its record (S-26).
//! No unit test: it runs git (S-5); refill_cone_tests covers the cone. Nagatha's integration group is to cover:
//! a folder whose fill was interrupted after its record was written is filled by the repeat; a repeat on a
//! filled folder leaves its files as they were and returns the same record.

use std::path::Path;

use gitio::Git;

use super::refill_cone::refill_cone;
use super::FolderError;
use crate::record::FolderRecord;

/// Sets the sparse paths of the worktree at `path` to refill_cone(`existing`), which checks them out, and
/// returns `existing`. Both git steps are safe to repeat, so a filled folder is left as it was.
pub(crate) fn refill_folder(git: &Git, path: &Path, existing: FolderRecord) -> Result<FolderRecord, FolderError> {
    git.sparse_set(path, &refill_cone(&existing)?).map_err(FolderError::Git)?;
    Ok(existing)
}
