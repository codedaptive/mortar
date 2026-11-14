//! range_diff_request: the git run that compares a reviewed series with its new version.

use crate::range_diff_ranges_request::range_diff_ranges_request;
use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `range-diff --no-color <base>..<reviewed> <base>..<new>`, read-only. `--no-color` keeps colour codes out even
/// where the user's config sets color.ui=always.
pub(crate) fn range_diff_request(base: &Oid, reviewed: &Oid, new: &Oid) -> (Vec<OsString>, RunOpts) {
    range_diff_ranges_request(base, reviewed, base, new)
}
