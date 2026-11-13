//! range_diff_request: the git run that compares a reviewed series with its new version.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `range-diff --no-color <base>..<reviewed> <base>..<new>`, read-only. `--no-color` keeps colour codes out even
/// where the user's config sets color.ui=always.
pub(crate) fn range_diff_request(base: &Oid, reviewed: &Oid, new: &Oid) -> (Vec<OsString>, RunOpts) {
    let base = base.as_str();
    (
        vec![
            "range-diff".into(),
            "--no-color".into(),
            format!("{base}..{}", reviewed.as_str()).into(),
            format!("{base}..{}", new.as_str()).into(),
        ],
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
