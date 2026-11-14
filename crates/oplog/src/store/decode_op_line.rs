//! One op log line back to its record alone, telling a torn write from a bad line.

use super::{decode_valued_line, LineError, OpRecord};

/// The record on `line` as decode_valued_line reads it, its values dropped: a line with a `values` member
/// decodes as the same record without it. Torn and Malformed as decode_valued_line gives them.
pub(crate) fn decode_op_line(line: &str) -> Result<OpRecord, LineError> {
    decode_valued_line(line).map(|v| v.record)
}
