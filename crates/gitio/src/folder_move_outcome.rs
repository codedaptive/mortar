//! FolderMoveOutcome: what moving a folder from one tree to another gave. A declaration only; read_folder_move_tests
//! exercises it.

use std::path::PathBuf;

/// The folder moved, or git refused and nothing changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderMoveOutcome {
    /// The folder's index and files now hold the new tree; local edits to paths the move does not change are kept.
    Moved,
    /// Nothing changed: a local edit, a staged change or an untracked, not ignored file stood in the
    /// way. These are the paths git named, in its order; git 2.56 names only the first.
    Refused(Vec<PathBuf>),
}
