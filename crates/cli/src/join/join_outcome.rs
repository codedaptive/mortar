//! What one `mr join` did, as join_report reads it.
//! A bare type; join_report_tests exercises it.

use ::join::BaseJoinAnswer;
use snapshot::SaveOutcome;

use super::PendingMove;

/// `request_ref` is the full ref name; `finished_move` the pending folder move (the last rebase's, a joined partner request's or an
/// undo's), which
/// this join finished or found already done (None when none was pending); `snapshot_written` whether the finished
/// move's snapshot, the formatter's snapshot before the save or the save's own snapshot step wrote a commit and moved
/// the snapshot ref; `save` the save of the folder's work before the join (its `written`: whether it moved the
/// request branch); `answer` the join's on the request's recorded base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinOutcome {
    pub request_ref: String,
    pub snapshot_written: bool,
    pub finished_move: Option<PendingMove>,
    pub save: SaveOutcome,
    pub answer: BaseJoinAnswer,
}
