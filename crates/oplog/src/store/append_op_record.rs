//! Appends one record, with no values, to the op log and syncs it to disk.
//! No unit test: it writes a real file through append_valued_record, whose header names what the
//! integration tests cover.

use std::path::Path;

use super::{append_valued_record, OpRecord, OplogError};

/// append_valued_record with no values: the line written is encode_op_record(r).
pub fn append_op_record(common: &Path, r: &OpRecord) -> Result<(), OplogError> {
    append_valued_record(common, r, &[])
}
