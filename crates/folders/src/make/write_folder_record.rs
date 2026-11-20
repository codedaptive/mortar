//! Writes a folder's record into its git dir, whole or not at all.
//! No unit test: it writes and renames real files; encode_folder_record_tests covers the text. The
//! integration group is to cover: the file holds the encoded record and one newline; writing again replaces it;
//! no temp file is left beside it.

use std::path::Path;

use super::FolderError;
use crate::record::{encode_folder_record, record_path, FolderRecord};

/// Writes encode_folder_record(`record`) and "\n" to `mortar-folder.json.tmp` beside record_path(`git_dir`),
/// then renames it over the record, so a reader finds the old record or the new one, never a torn one.
pub fn write_folder_record(git_dir: &Path, record: &FolderRecord) -> Result<(), FolderError> {
    let path = record_path(git_dir);
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, format!("{}\n", encode_folder_record(record)))
        .map_err(|source| FolderError::Io { path: temp.clone(), source })?;
    std::fs::rename(&temp, &path).map_err(|source| FolderError::Io { path, source })
}
