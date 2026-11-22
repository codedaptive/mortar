//! range_diff_answer: a RangeDiff re-review's answer from its text and the two commits' patch-ID reads. Pure: it
//! decides which failed read is the error and otherwise hands the IDs to same_patch.
//! range_diff_answer_tests.rs covers it.

use gitio::{GitError, PatchId};

use super::{same_patch, RereviewAnswer, RereviewError, RereviewPlan};

/// The answer carrying `text`, with `same_patch` decided from the two patch IDs. A failed read is the error, the
/// reviewed commit's first, then the new commit's, each as Git; it is never an answer with `same_patch` false.
pub(crate) fn range_diff_answer(
    plan: &RereviewPlan,
    text: String,
    reviewed_patch: Result<Option<PatchId>, GitError>,
    new_patch: Result<Option<PatchId>, GitError>,
) -> Result<RereviewAnswer, RereviewError> {
    let reviewed = reviewed_patch.map_err(RereviewError::Git)?;
    let new = new_patch.map_err(RereviewError::Git)?;
    Ok(RereviewAnswer { text, same_patch: same_patch(plan, reviewed.as_ref(), new.as_ref()) })
}
