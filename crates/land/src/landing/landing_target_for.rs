//! landing_target_for: the target of landing a squash onto a given commit, and the zone named in the squash.
//! No unit test: it runs git. The fast-forward choice in make_landing_target is covered by is_fast_forward_tests;
//! landing_target_for's own NoZone refusal has no test. The rest is covered by the
//! landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit and
//! a_conflicting_squash_moves_nothing_and_writes_no_state, and by the squash_zit would_merge_with cases
//! ordered_squashes_merge_cleanly, second_squash_conflicts_with_first_result and
//! preview_leaves_refs_and_reachable_objects_unchanged.

use crate::landing::{make_landing_target, LandError, LandingTarget};
use crate::squash::Squash;
use gitio::{Git, Oid};
use oplog::read_mortar_trailers;

/// Reads the squash commit's parents and the zone from its Mortar-Zone trailer (NoZone without one), then builds
/// the target of landing it onto `onto` with make_landing_target, and returns the target and the zone. It writes
/// only what make_landing_target writes: a merge commit when one is needed, nothing for a fast-forward, and only
/// merge-tree's tree objects when conflicted.
pub(crate) fn landing_target_for(git: &Git, onto: &Oid, squash: &Squash) -> Result<(LandingTarget, String), LandError> {
    let info = git.commit_info(&squash.commit).map_err(LandError::Git)?;
    let zone = read_mortar_trailers(git, &info.message)
        .map_err(LandError::Trailers)?
        .zone
        .ok_or_else(|| LandError::NoZone { commit: squash.commit.clone() })?;
    let target = make_landing_target(git, onto, squash, &info.parents, &zone)?;
    Ok((target, zone))
}
