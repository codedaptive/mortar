//! The op log as read, each record with its values, and the lines skipped as torn writes.
//! A bare type; decode_valued_log_tests exercises it.

use super::ValuedRecord;

/// `records` in file order; `torn_lines` holds the 1-based line numbers of torn writes that were skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValuedLog {
    pub records: Vec<ValuedRecord>,
    pub torn_lines: Vec<usize>,
}
