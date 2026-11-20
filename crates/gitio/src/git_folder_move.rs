//! Git::folder_move: move a folder's index and files from one tree to another (a two-tree merge), never its HEAD.
//! It only runs git, so it has no unit tests; join's tests/resolution/folder_trees.rs covers a move between two
//! trees answering Moved and keeping a local edit the move does not touch; folders' tests/folders/moves.rs covers
//! its refusals over an unstaged, a staged and an untracked file in the way.
//! Three git runs: a refresh of the index's stat data; the check, a dry run with sparse checkout off, because in a
//! sparse folder the real run writes over an untracked, not ignored file in the cone that the dry run refuses; then
//! the move, only when the check finds nothing in the way.

use crate::folder_move_requests::folder_move_requests;
use crate::read_folder_move::read_folder_move;
use crate::{FolderMoveOutcome, Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// Moves `folder`'s index and files from tree `old` to tree `new`, keeping local edits to paths the move does not
    /// change. A local edit, staged change or untracked, not ignored file in the way is Refused, with nothing
    /// changed; HEAD is never moved.
    pub fn folder_move(&self, folder: &Path, old: &Oid, new: &Oid) -> Result<FolderMoveOutcome, GitError> {
        let [refresh, check, read_tree] = folder_move_requests(folder, old, new)?;
        self.run(&refresh.0, &refresh.1)?;
        match read_folder_move(&self.run(&check.0, &check.1)?)? {
            FolderMoveOutcome::Moved => read_folder_move(&self.run(&read_tree.0, &read_tree.1)?),
            refused => Ok(refused),
        }
    }
}
