//! Deciding what a re-review of two versions of one change shows, from the two commits as git holds them.
//! plan_rereview_tests exercises every rule.
//!
//! Commits on different first parents are refused as BasesDiffer. That is a stopgap (S-77): gitio's range_diff
//! takes one base for both ranges, so it cannot compare a squash remade on a moved origin. When WO-gitio-15
//! (two-base range-diff) publishes, that case becomes a range-diff of reviewed^..reviewed against new^..new.

use gitio::{CommitInfo, Oid};

use super::{RereviewError, RereviewPlan};

/// The plan for `reviewed` (as `old`) against `new` (as `now`), checked in this order: same tree and parents is
/// NothingChanged, roots included; a commit with no parent is NoParent naming it (reviewed first); one first
/// parent is a RangeDiff on it; different first parents are BasesDiffer naming both.
pub(crate) fn plan_rereview(
    reviewed: &Oid,
    old: &CommitInfo,
    new: &Oid,
    now: &CommitInfo,
) -> Result<RereviewPlan, RereviewError> {
    if old.tree == now.tree && old.parents == now.parents {
        return Ok(RereviewPlan::NothingChanged);
    }
    let reviewed_parent = old.parents.first().ok_or_else(|| RereviewError::NoParent { commit: reviewed.clone() })?;
    let new_parent = now.parents.first().ok_or_else(|| RereviewError::NoParent { commit: new.clone() })?;
    if reviewed_parent == new_parent {
        return Ok(RereviewPlan::RangeDiff { base: reviewed_parent.clone() });
    }
    Err(RereviewError::BasesDiffer { reviewed_parent: reviewed_parent.clone(), new_parent: new_parent.clone() })
}
