//! What apply did with an op.
//! A bare type; no unit test. close_after_update_tests covers the Done and Failed records it carries,
//! find_done_record_tests the record Repeated carries. Covered by the crate's integration tests
//! (tests/oplog/apply.rs): each variant.

use crate::store::OpRecord;

/// Each variant carries the record that settles the op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The earlier `done` record for this key; nothing moved, nothing appended.
    Repeated(OpRecord),
    /// The refs moved; the appended `done` record.
    Done(OpRecord),
    /// A ref was not at its expected value and nothing moved; the appended `failed` record.
    Failed(OpRecord),
}
