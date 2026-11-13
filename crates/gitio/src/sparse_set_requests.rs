//! sparse_set_requests: the two git runs that set a folder's sparse paths and fill them.

use crate::no_nul::no_nul;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `sparse-checkout set --cone -- <paths>...`, then `checkout` with no arguments, both run in `folder`: setting the
/// cone only records it, and the checkout writes the cone's files (and the root's) into a folder added with
/// `--no-checkout`. An empty path or one holding a NUL is refused naming it. Neither is read-only: they write the
/// folder's sparse settings, index and files.
pub(crate) fn sparse_set_requests(
    folder: &Path,
    paths: &[String],
) -> Result<[(Vec<OsString>, RunOpts); 2], GitError> {
    let mut set: Vec<OsString> = ["sparse-checkout", "set", "--cone", "--"].map(OsString::from).to_vec();
    for path in paths {
        if path.is_empty() {
            return Err(GitError::refusal(Vec::new(), format!("not a usable sparse path (empty): {path:?}")));
        }
        no_nul("sparse path", path.as_bytes())?;
        set.push(path.into());
    }
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    Ok([(set, opts.clone()), (vec!["checkout".into()], opts)])
}
