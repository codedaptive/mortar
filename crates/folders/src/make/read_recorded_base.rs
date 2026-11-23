//! Reads what a folder stands on, its base and partners, from its git dir.
//! No unit test: read_record_and_partners does the reading. tests/folders/partner_base.rs covers a record with
//! partners, a record written without them (read back with none) and a record after need.

use std::path::Path;

use super::read_record_and_partners::read_record_and_partners;
use super::FolderError;
use crate::record::RecordedBase;

/// The base and partners of the record at record_path(`git_dir`), partners empty for a record without them; None
/// when there is no record file. A read failure is Io; text that is not a record is Record, both naming the file.
pub fn read_recorded_base(git_dir: &Path) -> Result<Option<RecordedBase>, FolderError> {
    Ok(read_record_and_partners(git_dir)?.map(|(record, partners)| RecordedBase { base: record.base, partners }))
}
