//! Git::snapshot_tree: save a folder's files as a tree without touching its index. It only runs git, so it has no
//! unit tests; Nagatha's integration test "snapshot_tree in a sparse folder keeping out-of-cone files, applying
//! excludes, leaving the folder's index unchanged and no temp index behind" is to cover it.

use crate::next_temp_index::next_temp_index;
use crate::oid_line::oid_line;
use crate::snapshot_tree_requests::snapshot_tree_requests;
use crate::{Git, GitError, Oid};

impl Git {
    /// The tree of every file in the folder (`self.repo`, which must be the folder's top), starting from `base` so
    /// files outside a sparse cone are kept, minus the `excludes` pathspecs. Writes objects; moves no ref.
    pub fn snapshot_tree(&self, base: &Oid, excludes: &[String]) -> Result<Oid, GitError> {
        let index = next_temp_index();
        let [read_tree, add, write_tree] = snapshot_tree_requests(base, excludes, &index.0)?;
        self.run(&read_tree.0, &read_tree.1)?;
        self.run(&add.0, &add.1)?;
        oid_line(&self.run(&write_tree.0, &write_tree.1)?.stdout)
    }
}
