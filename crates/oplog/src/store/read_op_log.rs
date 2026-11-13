//! Reads the op log file whole and decodes it.
//! No unit test: it reads a real file (S-5); decode_op_log_tests covers the decoding. Nagatha's
//! integration group should cover: a missing file reads as an empty log; records appended by
//! append_op_record read back equal and in order; a file cut mid-record reads with that line in
//! torn_lines; an unreadable path is an Io error naming the log's path.

use std::io::ErrorKind;
use std::path::Path;

use super::{decode_op_log, oplog_path, OpLog, OplogError};

/// The log at `<common>/mortar/oplog.jsonl`; a missing file is an empty log.
pub fn read_op_log(common: &Path) -> Result<OpLog, OplogError> {
    let path = oplog_path(common);
    match std::fs::read(&path) {
        Ok(bytes) => decode_op_log(&bytes),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(OpLog::default()),
        Err(source) => Err(OplogError::Io { path, source }),
    }
}
