//! Appends one record with its values to the op log and syncs it to disk (S-1), with no remote mark.
//! No unit test of its own: it writes a real file (S-5) through append_marked_record, whose header names
//! what the integration tests cover; each append adds exactly one line that read_valued_log returns with
//! its values.

use std::path::Path;

use super::{append_marked_record, OpRecord, OplogError};

/// append_marked_record(common, r, values, None): the line has no `remote` key.
pub fn append_valued_record(
    common: &Path,
    r: &OpRecord,
    values: &[(String, String)],
) -> Result<(), OplogError> {
    append_marked_record(common, r, values, None)
}
