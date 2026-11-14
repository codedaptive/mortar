//! One op record with the name/value pairs its caller set beside it.
//! A bare type; decode_valued_log_tests and crates/oplog/tests/oplog/valued_lines.rs exercise it.

use super::OpRecord;

/// `values` are the caller's name/value pairs in the caller's order, empty when none were set. oplog gives
/// the names no meaning: any strings, empty and repeated names included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValuedRecord {
    pub record: OpRecord,
    pub values: Vec<(String, String)>,
}
