//! Finishes a folder whose worktree exists: its record, then its files.
//! No unit test: it writes a file and runs git (S-5). Nagatha's integration group is to cover: after
//! fill_folder the folder's git dir holds the record and the folder holds the cone's folders and the root
//! files only.

use std::path::Path;

use gitio::Git;

use super::{write_folder_record, FolderError};
use crate::record::FolderRecord;

/// Writes `record` into the git dir of the worktree at `path`, then sets its sparse paths to `cone`, which
/// fills it. The record goes first, so a fill that is interrupted leaves a folder a repeat recognizes and fills
/// again through refill_folder.
pub(crate) fn fill_folder(
    git: &Git,
    path: &Path,
    record: &FolderRecord,
    cone: &[String],
) -> Result<(), FolderError> {
    let git_dir = git.git_dir(path).map_err(FolderError::Git)?;
    write_folder_record(&git_dir, record)?;
    git.sparse_set(path, cone).map_err(FolderError::Git)
}
