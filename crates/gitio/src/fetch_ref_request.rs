//! fetch_ref_request: the git run that copies one remote ref onto one local ref.

use crate::usable_ref_arg::usable_ref_arg;
use crate::usable_remote::usable_remote;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `fetch --no-tags --end-of-options <remote> +<src>:<dst>`. `+` forces dst onto src even when that is not a
/// fast-forward; `--no-tags` stops git following tags that point into what it fetched, so only dst is written;
/// `--end-of-options` keeps the remote from being read as an option. A write command (it moves dst), and only exit 0
/// is an answer. The remote, src and dst are refused first when git could not be given them (usable_remote,
/// usable_ref_arg).
pub(crate) fn fetch_ref_request(remote: &str, src: &str, dst: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_remote(remote)?;
    usable_ref_arg("source ref", src)?;
    usable_ref_arg("destination ref", dst)?;
    Ok((
        vec![
            "fetch".into(),
            "--no-tags".into(),
            "--end-of-options".into(),
            remote.into(),
            format!("+{src}:{dst}").into(),
        ],
        RunOpts::default(),
    ))
}
