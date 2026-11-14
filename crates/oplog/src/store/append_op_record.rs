//! Appends one record, with no values, to the op log and syncs it to disk (S-1).
//! No unit test: it writes a real file (S-5) through append_valued_record, whose header lists what
//! Nagatha's integration group should cover.

use std::path::Path;

use super::{append_valued_record, OpRecord, OplogError};

/// append_valued_record with no values: the line written is encode_op_record(r).
pub fn append_op_record(common: &Path, r: &OpRecord) -> Result<(), OplogError> {
    append_valued_record(common, r, &[])
}
