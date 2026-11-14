//! The re-review diff: what changed in a squash since it was reviewed. Read only: it moves no ref and takes no
//! lock. same_change and plan_rereview make every decision.
//! No unit test: it runs git (S-5). Covered by Nagatha's integration group (SPEC-land 02): "an edit after review
//! shows only the edit" and "a remake on a moved origin shows no change of its own", the second refused as
//! BasesDiffer until WO-gitio-15 (two-base range-diff) publishes (S-77).

use gitio::{Git, Oid};
use oplog::read_mortar_trailers;

use super::{plan_rereview, same_change, RereviewError, RereviewPlan};

/// What changed from `reviewed` to `new`, two versions of one squash: git's range-diff over their shared first
/// parent, or a line saying nothing changed when both have the same tree and parents.
///
/// Refused as NotSameChange when the two do not carry one Mortar-Change-Id (either or both without one
/// included), NoParent when either is a root commit with a different tree or parents, and BasesDiffer when
/// their first parents differ. Git when git fails, Trailers when a commit's trailers cannot be read.
pub fn rereview(git: &Git, reviewed: &Oid, new: &Oid) -> Result<String, RereviewError> {
    let old = git.commit_info(reviewed).map_err(RereviewError::Git)?;
    let now = git.commit_info(new).map_err(RereviewError::Git)?;
    let reviewed_trailers = read_mortar_trailers(git, &old.message).map_err(RereviewError::Trailers)?;
    let new_trailers = read_mortar_trailers(git, &now.message).map_err(RereviewError::Trailers)?;
    same_change(reviewed, &reviewed_trailers, new, &new_trailers)?;
    match plan_rereview(reviewed, &old, new, &now)? {
        RereviewPlan::NothingChanged => {
            Ok(format!("nothing changed: {new} has the same tree and parents as {reviewed}\n"))
        }
        RereviewPlan::RangeDiff { base } => git.range_diff(&base, reviewed, new).map_err(RereviewError::Git),
    }
}
