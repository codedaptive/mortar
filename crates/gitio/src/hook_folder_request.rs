//! hook_folder_request: the git run that says whether the repository is bare, to choose the folder commit hooks run in.

use crate::RunOpts;
use std::ffi::OsString;

/// `rev-parse --is-bare-repository`, read-only, in the Git value's repo. It prints "true\n" for a bare repository and
/// "false\n" otherwise; Git::hook_folder reads it with read_bare_flag.
pub(crate) fn hook_folder_request() -> (Vec<OsString>, RunOpts) {
    (
        ["rev-parse", "--is-bare-repository"].map(OsString::from).to_vec(),
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
