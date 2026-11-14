//! first_parent_list_request: the git run that lists a tip's first-parent line back to a base, oldest first.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `rev-list --reverse --first-parent --end-of-options <base>..<tip> --`, read-only: the commits of `tip`'s
/// first-parent line that `base` cannot reach, oldest first. `--first-parent` leaves out the commits a merge brought
/// in from its other parents. `--end-of-options` and `--` keep the range from being read as an option or a path.
pub(crate) fn first_parent_list_request(base: &Oid, tip: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "rev-list".into(),
            "--reverse".into(),
            "--first-parent".into(),
            "--end-of-options".into(),
            format!("{}..{}", base.as_str(), tip.as_str()).into(),
            "--".into(),
        ],
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
