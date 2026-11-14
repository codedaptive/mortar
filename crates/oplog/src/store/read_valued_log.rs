//! Reads the op log file whole and decodes it, each record with its values.
//! No unit test: it reads a real file (S-5); decode_valued_log_tests covers the decoding. Nagatha's
//! integration group should cover: a missing file reads as an empty log; records appended by
//! append_valued_record read back equal, with their values, and in order; a file cut mid-record reads
//! with that line in torn_lines; an unreadable path is an Io error naming the log's path.

use std::io::ErrorKind;
use std::path::Path;

use super::{decode_valued_log, oplog_path, OplogError, ValuedLog};

/// The log at `<common>/mortar/oplog.jsonl`; a missing file is an empty log.
pub fn read_valued_log(common: &Path) -> Result<ValuedLog, OplogError> {
    let path = oplog_path(common);
    match std::fs::read(&path) {
        Ok(bytes) => decode_valued_log(&bytes),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(ValuedLog::default()),
        Err(source) => Err(OplogError::Io { path, source }),
    }
}
