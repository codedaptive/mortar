//! same_patch: whether a re-review's two commits carry the same patch, decided from the plan and their patch IDs.
//! Pure; same_patch_tests.rs covers it.

use gitio::PatchId;

use super::RereviewPlan;

/// True when the plan is NothingChanged (the patch IDs are then not looked at), or when both patch IDs are present
/// and equal. None is an empty change, and an empty change on either side is never the same patch.
pub(crate) fn same_patch(plan: &RereviewPlan, reviewed_patch: Option<&PatchId>, new_patch: Option<&PatchId>) -> bool {
    match (plan, reviewed_patch, new_patch) {
        (RereviewPlan::NothingChanged, _, _) => true,
        (RereviewPlan::RangeDiff { .. }, Some(reviewed), Some(new)) => reviewed == new,
        (RereviewPlan::RangeDiff { .. }, _, _) => false,
    }
}
