//! blob_bytes_request: the git run that prints one blob's bytes.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `cat-file blob <blob>`, read-only. git prints the bytes exactly as stored; an id that is not a blob is its error.
pub(crate) fn blob_bytes_request(blob: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        ["cat-file", "blob", blob.as_str()].map(OsString::from).to_vec(),
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
