//! SavedState: whether a folder's files equal the target's last snapshot, and the files the size cap refused.
//! A declaration only. It lives in saved_state_type.rs, not saved_state.rs, because the function saved_state has
//! that file (the charter's naming rule would name both the same).

use crate::RefusedFile;

/// What saved_state answers for a folder and its snapshot target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedState {
    /// The folder's tree, built with the cap as snapshot builds it, equals the target's last snapshot's tree.
    pub saved: bool,
    /// Each new file (new against HEAD) over the size cap that the tree leaves out, with its size and the cap, as
    /// SnapshotOutcome.refused reports them (S-31).
    pub refused: Vec<RefusedFile>,
}
