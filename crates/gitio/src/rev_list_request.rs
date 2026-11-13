//! rev_list_request: the git run that lists the commits between two commits, oldest first.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `rev-list --reverse --topo-order --end-of-options <base>..<tip> --`, read-only: the commits reachable from `tip`
/// and not from `base`, parents before children. `--end-of-options` and `--` keep the range from being read as an
/// option or a path.
pub(crate) fn rev_list_request(base: &Oid, tip: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "rev-list".into(),
            "--reverse".into(),
            "--topo-order".into(),
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
