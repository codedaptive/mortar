//! Git::path_kind: what a path names in a tree. It only runs git, so it has no unit tests; the zone's integration
//! tests in crates/gitio/tests/path_kind cover it.

use crate::read_path_kind::read_path_kind;
use crate::tree_entry_request::tree_entry_request;
use crate::{Git, GitError, Oid, PathKind};
use std::path::Path;

impl Git {
    /// The kind of entry at `path`, taken from the root of `tree` (a tree or commit id): None when the tree has no such
    /// path; File for a blob (a regular or executable file, or a symlink, which is not followed); Folder for a tree;
    /// Gitlink for a commit entry (a submodule). Unlike tree_entry, a folder is an answer, not a refusal. It reads
    /// only and moves no ref.
    ///
    /// An empty path, or one holding a NUL, is refused. The path is matched literally (GIT_LITERAL_PATHSPECS=1), never
    /// as a pattern. A path not written as git stores it is answered as ls-tree answers it, not normalised: `file/`
    /// and `missing/` give None; `./file` is refused as another path, because git lists `file`; `/file` is git's own
    /// error; `dir/` lists the folder's children, so it is refused as another path, or gives None for an empty
    /// listing. Callers pass paths as git stores them.
    pub fn path_kind(&self, tree: &Oid, path: &Path) -> Result<Option<PathKind>, GitError> {
        let (args, opts) = tree_entry_request(tree, path)?;
        read_path_kind(&self.run(&args, &opts)?.stdout, path)
    }
}
