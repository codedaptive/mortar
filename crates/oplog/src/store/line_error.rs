//! Why one op log line is not a record: a torn write, or a bad line.
//! A bare type; decode_valued_line_tests, decode_op_line_tests, decode_valued_log_tests and
//! decode_op_log_tests exercise each variant.

/// `Torn`: the line stops before its JSON is complete. `Malformed`: anything else, with what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LineError {
    Torn,
    Malformed(String),
}
