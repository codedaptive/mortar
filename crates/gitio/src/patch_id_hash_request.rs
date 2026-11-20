//! patch_id_hash_request: the git run that hashes a patch text into its stable patch ID.

use crate::RunOpts;
use std::ffi::OsString;

/// `patch-id --stable`, read-only, with `diff` moved in as stdin. `--stable` also overrides the patchid.verbatim and
/// patchid.stable config.
pub(crate) fn patch_id_hash_request(diff: Vec<u8>) -> (Vec<OsString>, RunOpts) {
    (
        ["patch-id", "--stable"].map(OsString::from).to_vec(),
        RunOpts {
            stdin: Some(diff),
            read_only: true,
            ..RunOpts::default()
        },
    )
}
