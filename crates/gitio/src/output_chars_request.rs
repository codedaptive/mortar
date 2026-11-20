//! output_chars_request: the git run behind one BaselineQuery.

use crate::usable_path::usable_path;
use crate::usable_ref_arg::usable_ref_arg;
use crate::usable_worktree_path::usable_worktree_path;
use crate::{BaselineQuery, GitError, RunOpts};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// No pager and no colour whatever the user's config: `--no-pager` and `color.ui=never` come before the subcommand so
/// they hold for every kind. Other config still shapes the output, as it does for plain git.
const PLAIN: [&str; 3] = ["--no-pager", "-c", "color.ui=never"];

/// The args and opts for `query`, read-only, run in `folder`. Each kind ends `-- <paths>`: the `--` keeps a revision
/// from being read as a path and a path from being read as a revision. `--no-color` beats a user's `color.diff`;
/// `--no-ext-diff` keeps a configured external diff driver from running; `color.status=never` and `--long` keep colour
/// and the short form from changing what status prints. Checked before any args are built: an empty folder, a
/// revision or range that is empty, starts with '-' or holds ':' or a NUL, and an empty path or one holding a NUL
/// are refused naming them.
pub(crate) fn output_chars_request(folder: &Path, query: &BaselineQuery) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    let (head, paths): (Vec<&str>, &[PathBuf]) = match query {
        BaselineQuery::LogPatches { range, paths } => {
            usable_ref_arg("revision range", range)?;
            (vec!["log", "-p", "--no-color", "--no-ext-diff", range.as_str()], paths)
        }
        BaselineQuery::Diff { a, b, paths } => {
            usable_ref_arg("revision", a)?;
            usable_ref_arg("revision", b)?;
            (vec!["diff", "--no-color", "--no-ext-diff", a.as_str(), b.as_str()], paths)
        }
        BaselineQuery::DiffHead { paths } => (vec!["diff", "--no-color", "--no-ext-diff", "HEAD"], paths),
        BaselineQuery::Status { paths } => (vec!["-c", "color.status=never", "status", "--long"], paths),
    };
    for path in paths {
        usable_path(path)?;
    }
    let args = PLAIN
        .iter()
        .chain(&head)
        .chain(&["--"])
        .map(OsString::from)
        .chain(paths.iter().map(|path| path.as_os_str().to_os_string()))
        .collect();
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        read_only: true,
        ..RunOpts::default()
    };
    Ok((args, opts))
}
