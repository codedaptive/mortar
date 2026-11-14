//! The step after applying a branch op, from its outcome and, for a repeat, where the branch is now.

use gitio::Oid;
use oplog::ApplyOutcome;

use super::BranchStep;

/// Done is Ready. Repeated is answered from the op log alone, so `ref_now` (the branch's value read after it)
/// decides: at `tip` is Ready, absent is Remake (the branch was deleted after it was made), anywhere else is
/// Exists. Failed is Exists. `ref_now` is read only for Repeated; the other outcomes ignore it.
pub(crate) fn branch_step_after(outcome: &ApplyOutcome, ref_now: Option<&Oid>, tip: &Oid) -> BranchStep {
    match (outcome, ref_now) {
        (ApplyOutcome::Done(_), _) => BranchStep::Ready,
        (ApplyOutcome::Repeated(_), Some(now)) if now == tip => BranchStep::Ready,
        (ApplyOutcome::Repeated(_), None) => BranchStep::Remake,
        (ApplyOutcome::Repeated(_), Some(_)) | (ApplyOutcome::Failed(_), _) => BranchStep::Exists,
    }
}
