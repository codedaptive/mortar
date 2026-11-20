//! LandingTarget: the commit a landing moves integration to, or the files that stop it. A bare type; land's own
//! landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit and a_conflicting_squash_moves_nothing_and_writes_no_state
//! exercise each variant through land.

use gitio::Oid;
use std::path::PathBuf;

/// Commit: the squash itself (fast-forward) or the merge commit just written. Conflicted: the squash does not
/// merge into integration; these files conflict and no commit was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LandingTarget {
    Commit(Oid),
    Conflicted(Vec<PathBuf>),
}
