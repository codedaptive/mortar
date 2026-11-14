//! range_diff_ranges_request: the git run that compares one commit range with another.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `range-diff --no-color <base1>..<tip1> <base2>..<tip2>`, read-only. `--no-color` keeps colour codes out even
/// where the user's config sets color.ui=always. Every argument is an Oid, hex only, so no value can start with
/// `-` or carry an extra `..`: no option or range injection.
pub(crate) fn range_diff_ranges_request(
    base1: &Oid,
    tip1: &Oid,
    base2: &Oid,
    tip2: &Oid,
) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "range-diff".into(),
            "--no-color".into(),
            format!("{}..{}", base1.as_str(), tip1.as_str()).into(),
            format!("{}..{}", base2.as_str(), tip2.as_str()).into(),
        ],
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
