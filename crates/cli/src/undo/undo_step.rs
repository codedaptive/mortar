//! What undo does to the folder after oplog has reversed a step (S-142, S-143).
//! A bare type (S-7); choose_undo_tests, undo_report_tests and already_undone_report_tests exercise it.

/// SetIndex: a save was undone, so the folder's index is set to the tree of the commit its branch now holds and the
/// files stay as they are. MoveFolder: a join's rebase was undone, so the folder moves back to the tip before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoStep {
    SetIndex,
    MoveFolder,
}
