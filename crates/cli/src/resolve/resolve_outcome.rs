//! What one `mr resolve` did, as resolve_report reads it.
//! A bare type; resolve_report_tests exercises it.

use ::join::JoinAnswer;
use gitio::Oid;

use crate::join::PendingMove;

/// `request_ref` is the full ref name; `finished_move` the pending folder move (the last rebase's or an undo's), which
/// this resolve finished or found already done (None when none was pending); `snapshot_ref` and `snapshot_commit` the
/// snapshot of the folder taken before join resolve ran, and `snapshot_written` whether that snapshot or the finished
/// move's wrote a commit and moved the snapshot ref; `answer` join resolve's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveOutcome {
    pub request_ref: String,
    pub finished_move: Option<PendingMove>,
    pub snapshot_ref: String,
    pub snapshot_written: bool,
    pub snapshot_commit: Oid,
    pub answer: JoinAnswer,
}
