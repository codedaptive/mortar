//! What one `mortar resolve` did, as resolve_report reads it.
//! A bare type (S-7); crates/cli/tests/resolve_report_tests.rs exercises it.

use ::join::JoinAnswer;
use gitio::Oid;

use crate::join::PendingMove;

/// `request_ref` is the full ref name; `finished_move` the last rebase's folder move, which this resolve finished or
/// found already done (None when none was pending); `snapshot_ref` and `snapshot_commit` the snapshot of the folder
/// taken before join resolve ran; `answer` join resolve's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveOutcome {
    pub request_ref: String,
    pub finished_move: Option<PendingMove>,
    pub snapshot_ref: String,
    pub snapshot_commit: Oid,
    pub answer: JoinAnswer,
}
