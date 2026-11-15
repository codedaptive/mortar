//! consumed_record: the replay record a zone move consumes, from what was already read. consumed_record_tests
//! covers it.

use crate::after_publish::{SquashOfRead, ZoneCase};
use gitio::Oid;

/// The publish that moves its zone past the squash source a replay record names consumes that record, and the op
/// that moves the zone deletes it (S-118 (1)): Some(the record's value) when `case` is Advance or Replay and
/// `squash_of` is Found with a record; else None. Current moves nothing (ZoneMove::Current, S-117), so a record
/// still there then stays (folders' sweep_refs does not count it yet, S-118); a refusal
/// runs no op; a zone that did not publish has no record read.
pub(crate) fn consumed_record(case: &ZoneCase, squash_of: &SquashOfRead) -> Option<Oid> {
    match (case, squash_of) {
        (ZoneCase::Advance | ZoneCase::Replay { .. }, SquashOfRead::Found { record, .. }) => record.clone(),
        _ => None,
    }
}
