//! A folder record's JSON text back to the record.

use super::decode_record_and_partners::decode_record_and_partners;
use super::{FolderRecord, RecordError};

/// The record in `text`, the strict inverse of encode_record_and_partners: it accepts and refuses exactly what
/// decode_record_and_partners does. A request record's "partners" are accepted and dropped from the answer;
/// read_recorded_base returns them.
pub fn decode_folder_record(text: &str) -> Result<FolderRecord, RecordError> {
    decode_record_and_partners(text).map(|(record, _)| record)
}
