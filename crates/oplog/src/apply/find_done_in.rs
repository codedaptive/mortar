//! The item whose record shows an op with this key already ran, in a list of items that each hold a record.

use crate::store::{OpPhase, OpRecord};

/// The first item in `records` whose record (reached through `record`) has phase Done and this key. Intent
/// and failed records do not count: an op that failed or never closed may run again.
pub(crate) fn find_done_in<'a, R>(
    records: &'a [R],
    key: &str,
    record: fn(&R) -> &OpRecord,
) -> Option<&'a R> {
    records.iter().find(|item| {
        let r = record(item);
        r.phase == OpPhase::Done && r.key == key
    })
}
