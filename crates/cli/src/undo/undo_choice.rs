//! What choose_undo found in the op log for a folder (S-73, S-142).
//! A bare type (S-7); choose_undo_tests and undo_refusal_tests exercise it.

use super::UndoStep;

/// Nothing: no op of the folder's actor on its own refs or the zone branch. Step: the op at `index` in the records
/// read is undone, then the folder gets `step`. AlreadyUndone: the last such op is `undo_op`, an undo of
/// `reversed_op`, whose folder step is `step`. ZoneOwns: `op_id` moved the zone branch (a completed join). Resolved:
/// `op_id` is resolve's join-rebase, which carries a folder tree. NotAStep: `op_id`, of `kind`, is no step of this
/// folder's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoChoice {
    Nothing,
    Step { index: usize, step: UndoStep },
    AlreadyUndone { undo_op: String, reversed_op: String, step: UndoStep },
    ZoneOwns { op_id: String },
    Resolved { op_id: String },
    NotAStep { op_id: String, kind: String },
}
