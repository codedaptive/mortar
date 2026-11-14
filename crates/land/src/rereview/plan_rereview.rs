//! Deciding what a re-review of two versions of one change shows, from the two commits as git holds them.
//! plan_rereview_tests exercises every rule.

use gitio::{CommitInfo, Oid};

use super::{RereviewError, RereviewPlan};

/// The plan for `reviewed` (as `old`) against `new` (as `now`), checked in this order: same tree and parents is
/// NothingChanged, roots included; a commit with no parent is NoParent naming it (reviewed first); otherwise a
/// RangeDiff on each one's own first parent, whether the two share one or not.
pub(crate) fn plan_rereview(
    reviewed: &Oid,
    old: &CommitInfo,
    new: &Oid,
    now: &CommitInfo,
) -> Result<RereviewPlan, RereviewError> {
    if old.tree == now.tree && old.parents == now.parents {
        return Ok(RereviewPlan::NothingChanged);
    }
    let reviewed_base = old.parents.first().ok_or_else(|| RereviewError::NoParent { commit: reviewed.clone() })?;
    let new_base = now.parents.first().ok_or_else(|| RereviewError::NoParent { commit: new.clone() })?;
    Ok(RereviewPlan::RangeDiff { reviewed_base: reviewed_base.clone(), new_base: new_base.clone() })
}
