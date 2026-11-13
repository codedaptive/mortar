//! Git::folder_move: move a folder's index and files from one tree to another (a two-tree merge), never its HEAD.
//! It only runs git, so it has no unit tests; Nagatha's integration test "folder_move moving a folder between two
//! trees and keeping a local edit the move does not touch; refusing, with nothing changed, over a local edit, a staged
//! change and an untracked file in the way" is to cover it.
//! Two git runs: a refresh of the index's stat data, then the move.

use crate::folder_move_requests::folder_move_requests;
use crate::read_folder_move::read_folder_move;
use crate::{FolderMoveOutcome, Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// Moves `folder`'s index and files from tree `old` to tree `new`, keeping local edits to paths the move does not
    /// change. A local edit, staged change or untracked file in the way is Refused, with nothing changed; HEAD is never
    /// moved.
    pub fn folder_move(&self, folder: &Path, old: &Oid, new: &Oid) -> Result<FolderMoveOutcome, GitError> {
        let [refresh, read_tree] = folder_move_requests(folder, old, new)?;
        self.run(&refresh.0, &refresh.1)?;
        read_folder_move(&self.run(&read_tree.0, &read_tree.1)?)
    }
}
