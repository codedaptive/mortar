//! remote_push_urls_request: the git run that lists a remote's push URLs.

use crate::usable_remote::usable_remote;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `remote get-url --push --all --end-of-options <remote>`: `--end-of-options` keeps the name from being read as an
/// option. `get-url` takes configured remote names only: a URL or a path is git's exit 2 ("No such remote"), like any
/// unknown name. A read command; only exit 0 is an answer. The remote is refused first when git could not be given it
/// (usable_remote).
pub(crate) fn remote_push_urls_request(remote: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_remote(remote)?;
    Ok((
        vec![
            "remote".into(),
            "get-url".into(),
            "--push".into(),
            "--all".into(),
            "--end-of-options".into(),
            remote.into(),
        ],
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
