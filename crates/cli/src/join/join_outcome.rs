//! What one `mortar join` did, as join_report reads it.
//! A bare type (S-7); join_report_tests exercises it.

use ::join::JoinAnswer;
use snapshot::SaveOutcome;

use super::PendingMove;

/// `request_ref` is the full ref name; `finished_move` the last rebase's folder move, which this join finished or found
/// already done (None when none was pending); `save` the save of the folder's work before the join; `answer` the join's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinOutcome {
    pub request_ref: String,
    pub finished_move: Option<PendingMove>,
    pub save: SaveOutcome,
    pub answer: JoinAnswer,
}
