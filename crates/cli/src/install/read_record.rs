//! read_record: Reads install's record from disk.
//! No sibling test: it touches the disk. The install integration tests in install/integration/ cover it: uninstall.rs
//! (uninstall_without_record_changes_nothing for no record, the others for a record install wrote).

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::{decode_record, RecordItem};

/// None when there is no record; Err names the file and why it cannot be read or decoded.
pub fn read_record(path: &Path) -> Result<Option<Vec<RecordItem>>, String> {
    match fs::read_to_string(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("cannot read install's record {}: {error}", path.display())),
        Ok(text) => decode_record(&text)
            .map(Some)
            .map_err(|why| format!("install's record {} is not a record: {why}", path.display())),
    }
}
