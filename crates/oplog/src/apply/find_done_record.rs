//! The record that shows an op with this key already ran.

use crate::store::{OpPhase, OpRecord};

/// The first record in `records` with phase Done and this key. Intent and failed records do not count:
/// an op that failed or never closed may run again.
pub fn find_done_record<'a>(records: &'a [OpRecord], key: &str) -> Option<&'a OpRecord> {
    records
        .iter()
        .find(|r| r.phase == OpPhase::Done && r.key == key)
}
