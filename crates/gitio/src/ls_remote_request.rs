//! ls_remote_request: the git run that lists a remote's refs.

use crate::usable_ref_arg::usable_ref_arg;
use crate::usable_remote::usable_remote;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `ls-remote --end-of-options <remote> [<pattern>...]`: `--end-of-options` keeps the remote and patterns from being
/// read as options. No patterns lists every ref, HEAD included. A read command; only exit 0 is an answer (a pattern
/// matching nothing is exit 0 with empty output). The remote and each pattern are refused first when git could not
/// be given them (usable_remote, usable_ref_arg).
pub(crate) fn ls_remote_request(remote: &str, patterns: &[String]) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_remote(remote)?;
    for pattern in patterns {
        usable_ref_arg("ref pattern", pattern)?;
    }
    let mut args: Vec<OsString> = vec!["ls-remote".into(), "--end-of-options".into(), remote.into()];
    args.extend(patterns.iter().map(OsString::from));
    Ok((
        args,
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
