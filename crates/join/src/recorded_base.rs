//! recorded_base: the request base a join op recorded, before and after the join, read from the op's values.
//! recorded_base_tests covers both kinds in any phase, the first of repeated values taken, each cause of
//! NotRecorded and a garbled value of each key.

use crate::base_after_name::BASE_AFTER;
use crate::base_before_name::BASE_BEFORE;
use crate::decode_request_base::decode_request_base;
use crate::recorded_base_type::RecordedBase;
use crate::request_base::RequestBase;
use oplog::ValuedRecord;

/// The base `op` recorded under BASE_BEFORE and BASE_AFTER, for a join-rebase op (join_on_base's rebase, and
/// resolve's final op) or a join-zone-request op, in any phase: an intent, done and failed record of one op carry the
/// same values, so the caller picks the record. The first value of each key is taken. Another kind, or an op lacking
/// either value (ops written before join recorded the base), is NotRecorded, never an error. A value that does not
/// decode is Unreadable, its reason naming the key.
///
/// For the cli, whose pending-move finisher and undo are to put the folder record's base and partners back from
/// `before`, and finish a rewrite of the folder record to `after`.
pub fn recorded_base(op: &ValuedRecord) -> RecordedBase {
    if !matches!(op.record.kind.as_str(), "join-rebase" | "join-zone-request") {
        return RecordedBase::NotRecorded;
    }
    let (Some(before), Some(after)) = (value(op, BASE_BEFORE), value(op, BASE_AFTER)) else {
        return RecordedBase::NotRecorded;
    };
    match (decoded(BASE_BEFORE, before), decoded(BASE_AFTER, after)) {
        (Ok(before), Ok(after)) => RecordedBase::Recorded { before, after },
        (Err(reason), _) | (_, Err(reason)) => RecordedBase::Unreadable { reason },
    }
}

/// The first value `op` holds under `key`.
fn value<'a>(op: &'a ValuedRecord, key: &str) -> Option<&'a str> {
    op.values.iter().find(|(name, _)| name == key).map(|(_, text)| text.as_str())
}

/// `text` decoded, or the decoder's reason prefixed with `key`.
fn decoded(key: &str, text: &str) -> Result<RequestBase, String> {
    decode_request_base(text).map_err(|reason| format!("{key}: {reason}"))
}
