//! What apply_with_values did with an op, with the values of the record that settles it.
//! A bare type; no unit test. The zone tests crates/oplog/tests/oplog/values.rs and values_recover.rs exercise it.

use super::ApplyOutcome;

/// `values` belong to the record that settles the op: for Done and Failed, the values written (the caller's);
/// for Repeated, the done record's own values, what the first run wrote, not the caller's new ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValuedOutcome {
    pub outcome: ApplyOutcome,
    pub values: Vec<(String, String)>,
}
