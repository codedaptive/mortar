//! tree_entry_request: the git run that lists one path's entry in a tree.

use crate::usable_path::usable_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `ls-tree -z --full-tree --end-of-options <tree> -- <path>`, read-only, with GIT_LITERAL_PATHSPECS=1; the path goes
/// into argv as its bytes and is taken from the tree's root. ls-tree reads its path as a pathspec, and a path names an
/// entry, never a pattern: without the env a name like ":x" is read as pathspec magic (an empty answer, another
/// file's record or git's error). ls-tree, unlike `rev-parse <tree>:<path>`, names the entry's type, so a folder can
/// be told from a file. An empty path, or one holding a NUL, is refused naming it.
pub(crate) fn tree_entry_request(tree: &Oid, path: &Path) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_path(path)?;
    let mut args: Vec<OsString> =
        ["ls-tree", "-z", "--full-tree", "--end-of-options", tree.as_str(), "--"].map(OsString::from).to_vec();
    args.push(path.into());
    let opts = RunOpts {
        env: vec![("GIT_LITERAL_PATHSPECS".into(), "1".into())],
        read_only: true,
        ..RunOpts::default()
    };
    Ok((args, opts))
}
