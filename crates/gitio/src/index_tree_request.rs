//! index_tree_request: the git runs that find a folder's index, read a copy of it as a tree, and name its unmerged
//! paths.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// Three runs, all in `folder`. First `rev-parse --path-format=absolute --git-path index`, read-only: the absolute
/// path of the folder's own index (a linked worktree's is its own). The caller copies that file to `copy`. Second
/// `write-tree` with GIT_INDEX_FILE=`copy`: it writes the copy as tree objects and prints the top tree's id; not
/// read-only, as it writes objects and may rewrite the copy to cache the tree. Third, for when write-tree fails: `ls-files --unmerged -z
/// --full-name -- :/` with GIT_INDEX_FILE=`copy`, read-only, git's record of the copy's unmerged entries.
/// `--full-name` and the `:/` pathspec make it list the whole index with paths from the top of the worktree, as
/// write-tree refuses for the whole index; run in a subfolder without them, git lists only the paths under it,
/// relative to it. `copy` goes to git as its bytes and is not checked: gitio makes it with next_temp_index. An empty
/// folder path, or one holding a NUL, is refused naming it.
pub(crate) fn index_tree_request(folder: &Path, copy: &Path) -> Result<[(Vec<OsString>, RunOpts); 3], GitError> {
    usable_worktree_path(folder)?;
    let on_copy = vec![(OsString::from("GIT_INDEX_FILE"), copy.as_os_str().to_os_string())];
    let index_path = (
        ["rev-parse", "--path-format=absolute", "--git-path", "index"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            read_only: true,
            ..RunOpts::default()
        },
    );
    let write_tree = (
        vec![OsString::from("write-tree")],
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            env: on_copy.clone(),
            ..RunOpts::default()
        },
    );
    let unmerged = (
        ["ls-files", "--unmerged", "-z", "--full-name", "--", ":/"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            env: on_copy,
            read_only: true,
            ..RunOpts::default()
        },
    );
    Ok([index_path, write_tree, unmerged])
}
