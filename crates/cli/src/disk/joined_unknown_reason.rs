//! joined_unknown_reason: why a folder's joined answer is unknown, from the report its joined read failed with.

use crate::report::Report;

/// The refusal's lines joined with "; ": status's own words for why the read failed, on one line.
pub fn joined_unknown_reason(refusal: &Report) -> String {
    refusal.lines.join("; ")
}
