//! The op log as read: its records, and the lines skipped as torn writes.
//! A bare type; plain_log_tests exercises it.

use super::OpRecord;

/// `records` in file order; `torn_lines` holds the 1-based line numbers of torn writes that were skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpLog {
    pub records: Vec<OpRecord>,
    pub torn_lines: Vec<usize>,
}
