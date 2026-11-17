//! Reads the op log file whole and decodes it, each record with its values and remote mark.
//! No unit test: it reads a real file (S-5); decode_marked_log_tests covers the decoding. Nagatha's
//! integration group should cover: a missing file reads as an empty log; records appended by
//! append_marked_record read back equal, with their values and marks, and in order; a file cut mid-record
//! reads with that line in torn_lines; an unreadable path is an Io error naming the log's path.

use std::io::ErrorKind;
use std::path::Path;

use crate::store::{oplog_path, OplogError};

use super::{decode_marked_log, MarkedLog};

/// The log at `<common>/mortar/oplog.jsonl`; a missing file is an empty log.
pub fn read_marked_log(common: &Path) -> Result<MarkedLog, OplogError> {
    let path = oplog_path(common);
    match std::fs::read(&path) {
        Ok(bytes) => decode_marked_log(&bytes),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(MarkedLog::default()),
        Err(source) => Err(OplogError::Io { path, source }),
    }
}
