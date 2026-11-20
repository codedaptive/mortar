//! answer_rereview: the re-review diff with whether the patch is the same. Read only: it moves no ref and takes no
//! lock. It runs git only to gather inputs; same_change, plan_rereview and same_patch make every decision.
//! No unit test: it runs git. Its zone integration tests are in crates/land/tests/rereview_zit.rs.

use gitio::{Git, Oid};
use oplog::read_mortar_trailers;

use super::{plan_rereview, same_change, same_patch, RereviewAnswer, RereviewError, RereviewPlan};

/// What changed from `reviewed` to `new`, two versions of one squash, and whether `new` carries the same patch.
/// The text is git's range-diff of `reviewed^..reviewed` against `new^..new`, also when origin moved between them,
/// or a line saying nothing changed when both have the same tree and parents. `same_patch` is true for the same
/// tree and parents, or when both commits' patch IDs, each against its own first parent, are present and equal;
/// it says nothing about whether the review still holds on the new base.
///
/// Refused as NotSameChange when the two do not carry one Mortar-Change-Id (either or both without one
/// included), and NoParent when either is a root commit with a different tree or parents. Git when git fails,
/// reading a patch ID included (never an answer of "not the same patch"), Trailers when a commit's trailers
/// cannot be read.
pub fn answer_rereview(git: &Git, reviewed: &Oid, new: &Oid) -> Result<RereviewAnswer, RereviewError> {
    let old = git.commit_info(reviewed).map_err(RereviewError::Git)?;
    let now = git.commit_info(new).map_err(RereviewError::Git)?;
    let reviewed_trailers = read_mortar_trailers(git, &old.message).map_err(RereviewError::Trailers)?;
    let new_trailers = read_mortar_trailers(git, &now.message).map_err(RereviewError::Trailers)?;
    same_change(reviewed, &reviewed_trailers, new, &new_trailers)?;
    let plan = plan_rereview(reviewed, &old, new, &now)?;
    match &plan {
        RereviewPlan::NothingChanged => Ok(RereviewAnswer {
            text: format!("nothing changed: {new} has the same tree and parents as {reviewed}\n"),
            same_patch: same_patch(&plan, None, None),
        }),
        RereviewPlan::RangeDiff { reviewed_base, new_base } => {
            let text = git.range_diff_ranges(reviewed_base, reviewed, new_base, new).map_err(RereviewError::Git)?;
            let reviewed_patch = git.patch_id(reviewed).map_err(RereviewError::Git)?;
            let new_patch = git.patch_id(new).map_err(RereviewError::Git)?;
            Ok(RereviewAnswer { text, same_patch: same_patch(&plan, reviewed_patch.as_ref(), new_patch.as_ref()) })
        }
    }
}
