//! snapshot_tree_requests: the three git runs that save a folder's files as a tree through a separate index.

use crate::no_nul::no_nul;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `read-tree <base>`, then `add -A -- . :!<exclude>...`, then `write-tree`, each with GIT_INDEX_FILE=<index> so
/// the folder's own index is never read or written. Reading `base` first keeps files outside a sparse folder's cone
/// in the tree. Each exclude is a pathspec; an empty one, or one holding a NUL, is refused naming it. None is
/// read-only: each writes the index or objects.
pub(crate) fn snapshot_tree_requests(
    base: &Oid,
    excludes: &[String],
    index: &Path,
) -> Result<[(Vec<OsString>, RunOpts); 3], GitError> {
    let mut add: Vec<OsString> = ["add", "-A", "--", "."].map(OsString::from).to_vec();
    for exclude in excludes {
        if exclude.is_empty() {
            return Err(GitError::refusal(Vec::new(), format!("not a usable exclude (empty): {exclude:?}")));
        }
        no_nul("exclude", exclude.as_bytes())?;
        add.push(format!(":!{exclude}").into());
    }
    let opts = RunOpts {
        env: vec![("GIT_INDEX_FILE".into(), index.as_os_str().to_os_string())],
        ..RunOpts::default()
    };
    Ok([
        (vec!["read-tree".into(), base.as_str().into()], opts.clone()),
        (add, opts.clone()),
        (vec!["write-tree".into()], opts),
    ])
}
