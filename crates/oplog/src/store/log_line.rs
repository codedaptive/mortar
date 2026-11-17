//! The exact bytes one append adds to the op log, given the file's last byte.
//! log_line_tests (unit) covers a marked record with values on a torn tail; crates/oplog/tests/oplog/valued_lines.rs
//! (integration) covers an empty file, a file ending in '\n', a torn tail and a record with values, through
//! append_valued_record.

use super::marked::encode_marked_record;
use super::OpRecord;

/// encode_marked_record(r, values, remote) as one '\n'-ended line. `last_byte` is the log file's last
/// byte, None for an empty file. A file whose last byte is not '\n' was cut by a torn write, so a '\n'
/// goes first: the torn line stays torn and the new record starts on a line of its own.
pub(crate) fn log_line(
    r: &OpRecord,
    values: &[(String, String)],
    remote: Option<&str>,
    last_byte: Option<u8>,
) -> String {
    let lead = match last_byte {
        Some(b) if b != b'\n' => "\n",
        _ => "",
    };
    format!("{lead}{}\n", encode_marked_record(r, values, remote))
}
