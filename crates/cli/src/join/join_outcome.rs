//! What one `mortar join` did, as join_report reads it.
//! A bare type (S-7); crates/cli/tests/join_report_tests.rs exercises it.

use ::join::JoinAnswer;
use snapshot::SaveOutcome;

use super::PendingMove;

/// `request_ref` is the full ref name; `finished_move` the pending folder move (the last rebase's or an undo's), which
/// this join finished or found already done (None when none was pending); `save` the save of the folder's work before
/// the join; `answer` the join's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinOutcome {
    pub request_ref: String,
    pub finished_move: Option<PendingMove>,
    pub save: SaveOutcome,
    pub answer: JoinAnswer,
}
