//! is_saved: whether a folder's work is kept as a commit on its branch.

use gitio::Oid;

/// True when the folder's tree (`folder_tree`) equals its branch tip's tree (`tip_tree`). A tree equal only to the
/// snapshot ref's is not saved: a snapshot keeps the work, a save keeps it as a commit.
pub fn is_saved(folder_tree: &Oid, tip_tree: &Oid) -> bool {
    folder_tree == tip_tree
}
