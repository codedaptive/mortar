//! Finishes a folder whose worktree exists: its record and partners, then its files.
//! No unit test: it writes a file and runs git. The integration group is to cover: after
//! fill_folder the folder's git dir holds the record and the folder holds the cone's folders and the root
//! files only. tests/folders/root_files.rs covers a cone holding top-level files.

use std::path::Path;

use gitio::Git;

use super::{git_cone, write_folder_record, FolderError};
use crate::record::{FolderRecord, RequestPartner};

/// Writes `record` with `partners` (write_folder_record) into the git dir of the worktree at `path`, then sets its sparse paths to `cone` without its
/// top-level files (git_cone; cone mode checks those out anyway), which fills it. An empty result is set as
/// it is, which leaves the top-level files only. The record keeps `cone` as given. The record goes first, so a fill
/// that is interrupted leaves a folder a repeat recognizes and fills again through refill_folder.
pub(crate) fn fill_folder(
    git: &Git,
    path: &Path,
    record: &FolderRecord,
    partners: &[RequestPartner],
    cone: &[String],
) -> Result<(), FolderError> {
    let git_dir = git.git_dir(path).map_err(FolderError::Git)?;
    write_folder_record(&git_dir, record, partners)?;
    git.sparse_set(path, &git_cone(git, path, &record.base, cone)?).map_err(FolderError::Git)
}
