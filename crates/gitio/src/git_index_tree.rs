//! Git::index_tree: the tree a folder's own index holds, read from a temporary copy, with a typed refusal naming
//! unmerged paths. It only runs git and copies a file, so it has no unit tests; the zone's integration tests in
//! tests/index_tree cover it.

use crate::index_tree_request::index_tree_request;
use crate::next_temp_index::next_temp_index;
use crate::oid_line::oid_line;
use crate::path_line::path_line;
use crate::read_unmerged_paths::read_unmerged_paths;
use crate::{Git, GitError, IndexTreeError, Oid};
use std::io::ErrorKind;
use std::path::Path;

impl Git {
    /// The id of the tree the index of `folder`, a worktree of this repository, holds, by `write-tree` on a temporary
    /// copy of that index. Only the index is read: unstaged edits are not in the tree, and the same index gives the
    /// same id. No ref moves and the objects written are trees.
    ///
    /// The folder's index file is only read, once, by the copy; git replaces an index by renaming a new file over it,
    /// so the copy is one whole version. Its index.lock is never taken or created, so a lock another git holds does
    /// not stop the read. The copy is a temporary file gitio removes on every return, and on a panic; write-tree
    /// takes `<copy>.lock` beside it and either renames it over the copy (when it caches the tree there) or removes
    /// it, so nothing else is left. The index is found by `rev-parse --git-path index` in the folder, so a linked
    /// worktree's copy is of its own index. Settled by capture (git 2.56): a folder with no index file yet gives
    /// git's empty tree (no copy is made; git reads the absent copy as an empty index, as it does the folder's, and
    /// the file it creates there is removed); a split index reads correctly from the copy, as git finds its shared
    /// part through the folder's git dir; a sparse index gives the same tree from the copy as from the folder.
    ///
    /// The runs take this Git value's env after gitio removes GIT_INDEX_FILE, then their own. A GIT_INDEX_FILE set on
    /// this Git value names the index that is copied, as `--git-path index` answers it; write-tree and ls-files then
    /// read the copy.
    ///
    /// On an index with unmerged entries write-tree exits 128 with an `<path>: unmerged (<oid>)` line per stage and
    /// `fatal: git-write-tree: error building trees` on stderr. That text is not read: when write-tree fails, `ls-files
    /// --unmerged` on the copy names the paths from git's own record, and a non-empty list is Unmerged, each path
    /// once, full from the top of the worktree even when `folder` is a subfolder of it, in git's order. Every other
    /// refusal is Git: an empty or NUL-holding folder path (git does not run), a failed index lookup or an answer
    /// that is not one absolute path, a copy that fails for any reason but a missing index file (naming both paths
    /// and the error), an answer that is not one id line, and any write-tree failure with no unmerged paths listed
    /// (or whose listing itself fails), carrying write-tree's exit and stderr.
    pub fn index_tree(&self, folder: &Path) -> Result<Oid, IndexTreeError> {
        let copy = next_temp_index();
        let [(index_args, index_opts), (write_args, write_opts), (list_args, list_opts)] =
            index_tree_request(folder, &copy.0).map_err(IndexTreeError::Git)?;
        copy_index(
            &path_line(&self.run(&index_args, &index_opts).map_err(IndexTreeError::Git)?.stdout)
                .map_err(IndexTreeError::Git)?,
            &copy.0,
        )
        .map_err(IndexTreeError::Git)?;
        let refused = match self.run(&write_args, &write_opts) {
            Ok(output) => return oid_line(&output.stdout).map_err(IndexTreeError::Git),
            Err(error) => error,
        };
        match self.run(&list_args, &list_opts).ok().and_then(|listed| read_unmerged_paths(&listed.stdout).ok()) {
            Some(paths) if !paths.is_empty() => Err(IndexTreeError::Unmerged { paths }),
            _ => Err(IndexTreeError::Git(refused)),
        }
    }
}

/// Copies the folder's index to the temporary path. A missing index makes no copy: git then reads the absent copy as
/// an empty index, as it reads the folder's own missing index.
fn copy_index(index: &Path, copy: &Path) -> Result<(), GitError> {
    match std::fs::copy(index, copy) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(GitError::refusal(
            Vec::new(),
            format!("cannot copy the index {} to {}: {error}", index.display(), copy.display()),
        )),
    }
}
