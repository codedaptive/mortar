//! The re-review diff: what changed in a squash since it was reviewed. Read only: it moves no ref and takes no
//! lock. It returns answer_rereview's text; answer_rereview gathers the inputs and same_change, plan_rereview and
//! same_patch make every decision.
//! No unit test: it runs git. Its zone integration tests are in crates/land/tests/rereview_zit.rs.

use gitio::{Git, Oid};

use super::{answer_rereview, RereviewError};

/// What changed from `reviewed` to `new`, two versions of one squash: git's range-diff of `reviewed^..reviewed`
/// against `new^..new`, also when origin moved between them, or a line saying nothing changed when both have the
/// same tree and parents.
///
/// Refused as NotSameChange when the two do not carry one Mortar-Change-Id (either or both without one
/// included), and NoParent when either is a root commit with a different tree or parents. Git when git fails,
/// Trailers when a commit's trailers cannot be read.
pub fn rereview(git: &Git, reviewed: &Oid, new: &Oid) -> Result<String, RereviewError> {
    answer_rereview(git, reviewed, new).map(|answer| answer.text)
}
