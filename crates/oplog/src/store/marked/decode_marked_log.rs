//! The op log file's bytes back to its records, each with its values and remote mark.
//! Its decisions are proved by decode_marked_log_tests.

use crate::store::{LineError, OplogError};

use super::{decode_marked_line, MarkedLog};

/// Splits on '\n' and decodes each line; empty lines are skipped. A torn write (JSON that stops early, or
/// UTF-8 cut short at the line's end) is skipped and its 1-based line number kept in torn_lines: it is a
/// record that was never acknowledged. A last line without '\n' that decodes is kept. Any other bad line
/// is Malformed with its line number.
pub(crate) fn decode_marked_log(bytes: &[u8]) -> Result<MarkedLog, OplogError> {
    let mut log = MarkedLog::default();
    for (index, segment) in bytes.split(|b| *b == b'\n').enumerate() {
        let line = index + 1;
        if segment.is_empty() {
            continue;
        }
        let decoded = match std::str::from_utf8(segment) {
            Ok(text) => decode_marked_line(text),
            Err(e) if e.error_len().is_none() => Err(LineError::Torn),
            Err(e) => Err(LineError::Malformed(format!(
                "not UTF-8 at byte {}",
                e.valid_up_to()
            ))),
        };
        match decoded {
            Ok(record) => log.records.push(record),
            Err(LineError::Torn) => log.torn_lines.push(line),
            Err(LineError::Malformed(reason)) => {
                return Err(OplogError::Malformed {
                    line: Some(line),
                    reason,
                })
            }
        }
    }
    Ok(log)
}
