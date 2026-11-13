//! attr_check_request: the git run that reads the `merge` attribute of paths from a commit's attributes.

use crate::usable_path::usable_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::PathBuf;

/// `--attr-source=<commit> check-attr -z merge -- <paths>...`, read-only. `--attr-source` is a git option, not a
/// check-attr one, so it comes before the command: the attributes are read from that commit, never the folder's.
/// Paths go into argv as their bytes. An empty list (git exits 129, "No file specified") is refused; an empty path
/// or one holding a NUL is refused naming it.
pub(crate) fn attr_check_request(commit: &Oid, paths: &[PathBuf]) -> Result<(Vec<OsString>, RunOpts), GitError> {
    if paths.is_empty() {
        return Err(GitError::refusal(Vec::new(), "no paths to check"));
    }
    let mut args: Vec<OsString> = vec![
        format!("--attr-source={}", commit.as_str()).into(),
        "check-attr".into(),
        "-z".into(),
        "merge".into(),
        "--".into(),
    ];
    for path in paths {
        usable_path(path)?;
        args.push(path.into());
    }
    let opts = RunOpts {
        read_only: true,
        ..RunOpts::default()
    };
    Ok((args, opts))
}
