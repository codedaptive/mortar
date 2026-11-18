//! Reads `<common>/mortar/oplog.jsonl` and gives one op's moves while its intent is open, for the guard hook.
//! No unit test: it reads a real file (S-5); open_intent_moves_tests covers the
//! decision. Nagatha's integration group should cover: on a real log written by append_op_record, an open
//! intent's moves come back; after close_record's Done or Failed record is appended, None; a missing log
//! file gives None; an unreadable log path is an Io error naming the log's path; a torn last line (a record
//! cut mid-write) is skipped, so an earlier complete intent still comes back open.

use std::path::Path;

use super::open_intent_moves;
use crate::store::{read_op_log, OplogError, RefMove};

/// `op_id`'s planned moves when its intent is open in the log at `common`, else None.
pub fn open_intent(common: &Path, op_id: &str) -> Result<Option<Vec<RefMove>>, OplogError> {
    Ok(open_intent_moves(&read_op_log(common)?.records, op_id))
}
