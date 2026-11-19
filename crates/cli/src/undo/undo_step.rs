//! What undo does to the folder after oplog has reversed a step.
//! A bare type; choose_undo_tests, undo_report_tests and already_undone_report_tests exercise it.

/// SetIndex: a save was undone, so the folder's index is set to the tree of the commit its branch now holds and the
/// files stay as they are. MoveFolder: a join's rebase was undone, so the folder moves back to the tip before it.
/// RestoreJoin: a resolve's final join-rebase was undone, so the paused join is put back (join's restore_join_state)
/// and then the index is set to the branch's tree, the files kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoStep {
    SetIndex,
    MoveFolder,
    RestoreJoin,
}
