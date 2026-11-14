//! Reads the op log file whole and decodes it, without the records' values.
//! No unit test: it reads a real file (S-5) through read_valued_log; plain_log_tests covers dropping the
//! values. Nagatha's integration group should cover: a log whose lines carry values reads as the same
//! records without them.

use std::path::Path;

use super::{plain_log, read_valued_log, OpLog, OplogError};

/// The log read_valued_log reads at `<common>/mortar/oplog.jsonl`, each record's values dropped by
/// plain_log; a missing file is an empty log.
pub fn read_op_log(common: &Path) -> Result<OpLog, OplogError> {
    read_valued_log(common).map(plain_log)
}
