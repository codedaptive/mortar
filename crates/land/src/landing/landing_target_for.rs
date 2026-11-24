//! landing_target_for: the target of landing a squash onto a given commit, and the zone named in the squash.
//! No unit test: it runs git. The fast-forward choice in make_landing_target is covered by is_fast_forward_tests;
//! squash_zone's NoZone refusal has no test. The rest is covered by the
//! landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit,
//! a_conflicting_squash_moves_nothing_and_writes_no_state, three_zones_append_in_order and
//! conflict_keeps_batch_and_next_zone_can_append, and by the squash_zit would_merge_with cases
//! ordered_squashes_merge_cleanly, second_squash_conflicts_with_first_result and
//! preview_leaves_refs_and_reachable_objects_unchanged.

use crate::landing::{make_landing_target, squash_zone, LandError, LandingTarget};
use crate::squash::Squash;
use gitio::{Git, Oid};

/// Reads the zone from the squash commit (squash_zone, NoZone without one) and the commit's parents, then builds
/// the target of landing it onto `onto` with make_landing_target, and returns the target and the zone. It writes
/// only what make_landing_target writes: a merge commit when one is needed, nothing for a fast-forward, and only
/// merge-tree's tree objects when conflicted.
pub(crate) fn landing_target_for(git: &Git, onto: &Oid, squash: &Squash) -> Result<(LandingTarget, String), LandError> {
    let zone = squash_zone(git, squash)?;
    let parents = git.commit_info(&squash.commit).map_err(LandError::Git)?.parents;
    let target = make_landing_target(git, onto, squash, &parents, &zone)?;
    Ok((target, zone))
}
