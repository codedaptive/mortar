//! One op log line back to a record and its values, telling a torn write from a bad line.
//! Its one caller is decode_op_line (public route decode_op_record), which drops the values, so no public
//! entry point reaches the values it keeps. decode_valued_log decodes through decode_marked_log, not through
//! this function; crates/oplog/tests/oplog/valued_lines.rs and crates/oplog/tests/oplog/marked_lines.rs prove
//! that reader's outcomes.

use super::marked::decode_marked_line;
use super::{LineError, ValuedRecord};

/// The record and values on `line` as decode_marked_line reads it, the remote mark dropped: a line with a
/// `remote` key decodes as the same record and values without it. `result` and `values` optional (no
/// `values` key: no values). Torn and Malformed as decode_marked_line gives them.
pub(crate) fn decode_valued_line(line: &str) -> Result<ValuedRecord, LineError> {
    decode_marked_line(line).map(|m| ValuedRecord {
        record: m.record,
        values: m.values,
    })
}
