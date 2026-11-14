//! The op log file's bytes back to its records, each with its values.

use super::{decode_valued_line, LineError, OplogError, ValuedLog};

/// Splits on '\n' and decodes each line; empty lines are skipped. A torn write (JSON that stops early, or
/// UTF-8 cut short at the line's end) is skipped and its 1-based line number kept in torn_lines: it is a
/// record that was never acknowledged, and an intent whose closing record tore stays open for recover to
/// settle from the refs. A last line without '\n' that decodes is kept. Any other bad line is Malformed
/// with its line number.
pub fn decode_valued_log(bytes: &[u8]) -> Result<ValuedLog, OplogError> {
    let mut log = ValuedLog::default();
    for (index, segment) in bytes.split(|b| *b == b'\n').enumerate() {
        let line = index + 1;
        if segment.is_empty() {
            continue;
        }
        let decoded = match std::str::from_utf8(segment) {
            Ok(text) => decode_valued_line(text),
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
