//! The op log as read, each record with its values and remote mark, and the lines skipped as torn writes.
//! A bare type; decode_marked_log_tests exercises it through decode_marked_log.

use super::MarkedRecord;

/// `records` in file order; `torn_lines` holds the 1-based line numbers of torn writes that were skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkedLog {
    pub records: Vec<MarkedRecord>,
    pub torn_lines: Vec<usize>,
}
