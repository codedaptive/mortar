//! Reads the op log and returns the records a history filter keeps.
//! No unit test: it reads the log file (S-5); filter_history_tests covers the filtering. Nagatha's
//! integration group should cover: a missing log returns no records; after appends, each filter (ref,
//! key, actor) returns the matching records equal to what was appended and in log order, intents and
//! their closing records both; torn lines are skipped; an unreadable log path is an Io error naming it.

use std::path::Path;

use super::{filter_history, HistoryFilter};
use crate::store::{read_op_log, OpRecord, OplogError};

/// The records of `<common>/mortar/oplog.jsonl` that match `f`, in log order.
pub fn history(common: &Path, f: &HistoryFilter) -> Result<Vec<OpRecord>, OplogError> {
    let log = read_op_log(common)?;
    Ok(filter_history(&log.records, f).into_iter().cloned().collect())
}
