//! list_refs_request: the git run that lists refs with their ids.

use crate::no_nul::no_nul;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `for-each-ref --format=%(objectname) %(refname) --end-of-options <prefix>`; an empty prefix is left out, which
/// lists every ref. `--end-of-options` keeps a prefix starting with `-` from being read as an option. A prefix
/// holding a NUL is refused naming it.
pub(crate) fn list_refs_request(prefix: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    no_nul("ref prefix", prefix.as_bytes())?;
    let mut args = ["for-each-ref", "--format=%(objectname) %(refname)", "--end-of-options"]
        .map(OsString::from)
        .to_vec();
    if !prefix.is_empty() {
        args.push(OsString::from(prefix));
    }
    Ok((
        args,
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
