//! The exact bytes one append adds to the op log, given the file's last byte.
//! Pure; log_line_tests covers an empty file, a file ending in '\n', a torn tail and a record with values.

use super::{encode_valued_record, OpRecord};

/// encode_valued_record(r, values) as one '\n'-ended line. `last_byte` is the log file's last byte, None
/// for an empty file. A file whose last byte is not '\n' was cut by a torn write, so a '\n' goes first:
/// the torn line stays torn and the new record starts on a line of its own.
pub(crate) fn log_line(r: &OpRecord, values: &[(String, String)], last_byte: Option<u8>) -> String {
    let lead = match last_byte {
        Some(b) if b != b'\n' => "\n",
        _ => "",
    };
    format!("{lead}{}\n", encode_valued_record(r, values))
}
