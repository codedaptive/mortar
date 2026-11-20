//! Reads a folder's record from its git dir.
//! No unit test: it reads a real file; decode_folder_record_tests covers the decoding. The
//! integration group is to cover: a record written by write_folder_record reads back equal; a git dir with no
//! record reads None; a record file holding text that is not a record refuses with FolderError::Record naming
//! the file.

use std::io::ErrorKind;
use std::path::Path;

use super::FolderError;
use crate::record::{decode_folder_record, record_path, FolderRecord};

/// The record at record_path(`git_dir`); None when there is no record file. A read failure is Io; text that is
/// not a record is Record, both naming the file.
pub fn read_folder_record(git_dir: &Path) -> Result<Option<FolderRecord>, FolderError> {
    let path = record_path(git_dir);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(FolderError::Io { path, source }),
    };
    match decode_folder_record(&text) {
        Ok(record) => Ok(Some(record)),
        Err(error) => Err(FolderError::Record { path, error }),
    }
}
