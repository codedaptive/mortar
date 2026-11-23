//! Reads a folder's record and its partners from its git dir.
//! No unit test: it reads a real file; decode_record_and_partners_tests covers the decoding.
//! tests/folders/partner_base.rs covers a record with partners and one without reading back through
//! read_recorded_base and read_folder_record. The integration group is to cover: a git dir with no record reads
//! None; a record file holding text that is not a record refuses with FolderError::Record naming the file.

use std::io::ErrorKind;
use std::path::Path;

use super::FolderError;
use crate::record::{decode_record_and_partners, record_path, FolderRecord, RequestPartner};

/// The record at record_path(`git_dir`) and its partners (empty when the record names none); None when there is
/// no record file. A read failure is Io; text that is not a record is Record, both naming the file.
pub(crate) fn read_record_and_partners(
    git_dir: &Path,
) -> Result<Option<(FolderRecord, Vec<RequestPartner>)>, FolderError> {
    let path = record_path(git_dir);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(FolderError::Io { path, source }),
    };
    match decode_record_and_partners(&text) {
        Ok(read) => Ok(Some(read)),
        Err(error) => Err(FolderError::Record { path, error }),
    }
}
