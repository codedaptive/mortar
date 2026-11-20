//! changed_files_request: the git run that lists the paths differing between two commits.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `diff --name-only -z --no-renames <a> <b>`, read-only. `--no-renames` lists both sides of a rename; without it
/// git lists only the new name. `-z` keeps paths holding newlines or quotes as they are.
pub(crate) fn changed_files_request(a: &Oid, b: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        ["diff", "--name-only", "-z", "--no-renames", a.as_str(), b.as_str()].map(OsString::from).to_vec(),
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
