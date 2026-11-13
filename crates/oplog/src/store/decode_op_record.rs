//! One op log line back to a record, as a public refusal.

use super::{decode_op_line, LineError, OpRecord, OplogError};

/// The record on `line` (no trailing newline). Every refusal, a torn line included, is Malformed with no
/// line number; decode_op_log is the reader that skips torn lines.
pub fn decode_op_record(line: &str) -> Result<OpRecord, OplogError> {
    decode_op_line(line).map_err(|e| OplogError::Malformed {
        line: None,
        reason: match e {
            LineError::Torn => "the record stops before its JSON is complete".to_string(),
            LineError::Malformed(reason) => reason,
        },
    })
}
