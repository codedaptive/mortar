//! SavedState: whether a folder's files equal the target's last snapshot, the folder's tree it compared, and the
//! files the size cap refused.
//! A declaration only. It lives in saved_state_type.rs, not saved_state.rs, because the function saved_state has
//! that file (each file is named after its symbol, which would name both the same).

use crate::RefusedFile;
use gitio::Oid;

/// What saved_state answers for a folder and its snapshot target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedState {
    /// The folder's tree, built with the cap as snapshot builds it, equals the target's last snapshot's tree.
    pub saved: bool,
    /// The folder's tree as saved_state builds it: folder_tree with the caller's excludes and size cap, so new files
    /// over the cap are left out. It is built without a formatter; snapshot_formatted runs the caller's formatter
    /// before building its tree, so with a formatter a snapshot's tree can differ.
    pub tree: Oid,
    /// Each new file (new against HEAD) over the size cap that the tree leaves out, with its size and the cap, as
    /// SnapshotOutcome.refused reports them.
    pub refused: Vec<RefusedFile>,
}
