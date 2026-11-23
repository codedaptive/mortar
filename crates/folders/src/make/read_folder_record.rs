//! Reads a folder's record from its git dir, without its partners.
//! No unit test: read_record_and_partners does the reading, and its header says what covers it.

use std::path::Path;

use super::read_record_and_partners::read_record_and_partners;
use super::FolderError;
use crate::record::FolderRecord;

/// The record at record_path(`git_dir`), as read_record_and_partners reads it, its partners left out; None when
/// there is no record file. A read failure is Io; text that is not a record is Record, both naming the file.
pub fn read_folder_record(git_dir: &Path) -> Result<Option<FolderRecord>, FolderError> {
    Ok(read_record_and_partners(git_dir)?.map(|(record, _)| record))
}
