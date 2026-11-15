//! Fills a folder a same-record repeat found, unless its index has unmerged entries, then returns its record
//! (S-26, S-36).
//! No unit test: it runs git (S-5); skip_refill_tests covers the decision to skip, refill_cone_tests the cone and
//! cone_folders_tests the top-level files dropped from it; tests/folders/root_files.rs covers a refill with them.
//! Nagatha's integration group is to cover: a folder whose fill was interrupted after its record was written is
//! filled by the repeat; a repeat on a filled folder leaves its files as they were and returns the same record; a
//! same-record repeat during a conflicted rebase returns the record and leaves the unmerged index, files and rebase
//! state as they were.

use std::path::Path;

use gitio::Git;

use super::refill_cone::refill_cone;
use super::skip_refill::skip_refill;
use super::{git_cone, FolderError};
use crate::record::FolderRecord;

/// Reads the status of the worktree at `path` (read-only) and returns `existing`. Unless the index has unmerged
/// entries (S-36), it first sets the sparse paths to refill_cone(`existing`) without its top-level files (git_cone,
/// S-126; cone mode checks those out anyway), which also checks the cone out and is safe to repeat, so a filled
/// folder is left as it was. A skipped refill leaves the index, files, replay state, record and op log as they were.
pub(crate) fn refill_folder(git: &Git, path: &Path, existing: FolderRecord) -> Result<FolderRecord, FolderError> {
    if skip_refill(&git.status_paths(path).map_err(FolderError::Git)?) {
        return Ok(existing);
    }
    let cone = git_cone(git, path, &existing.base, &refill_cone(&existing)?)?;
    git.sparse_set(path, &cone).map_err(FolderError::Git)?;
    Ok(existing)
}
