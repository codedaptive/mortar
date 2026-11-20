//! head_requests: the two git runs that read a folder's HEAD.

use crate::rev_parse_request::rev_parse_request;
use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `symbolic-ref -q HEAD` (the branch; exit 1 is a detached HEAD), then rev_parse_request("HEAD") (the commit; exit
/// 1 is an unborn HEAD, where a plain `rev-parse HEAD` would exit 128), both read-only and run in `folder`. An empty
/// folder path, or one holding a NUL, is refused naming it.
pub(crate) fn head_requests(folder: &Path) -> Result<[(Vec<OsString>, RunOpts); 2], GitError> {
    usable_worktree_path(folder)?;
    let branch = (
        ["symbolic-ref", "-q", "HEAD"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            answer_codes: vec![1],
            read_only: true,
            ..RunOpts::default()
        },
    );
    let (args, mut opts) = rev_parse_request("HEAD")?;
    opts.cwd = Some(folder.to_path_buf());
    Ok([branch, (args, opts)])
}
