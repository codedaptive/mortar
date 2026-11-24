//! squash_zone: the zone a squash commit names in its Mortar-Zone trailer.
//! No unit test: it runs git. Covered, through land_batch, by the landing_zit tests three_zones_append_in_order and
//! duplicate_zone_reports_its_member_and_changes_nothing, and through landing_target_for by the squash_zit
//! would_merge_with cases ordered_squashes_merge_cleanly, second_squash_conflicts_with_first_result and
//! preview_leaves_refs_and_reachable_objects_unchanged. Its NoZone refusal has no test.

use crate::landing::LandError;
use crate::squash::Squash;
use gitio::Git;
use oplog::read_mortar_trailers;

/// The zone in the squash commit's Mortar-Zone trailer; NoZone when the commit carries none. Writes nothing.
pub(crate) fn squash_zone(git: &Git, squash: &Squash) -> Result<String, LandError> {
    let info = git.commit_info(&squash.commit).map_err(LandError::Git)?;
    read_mortar_trailers(git, &info.message)
        .map_err(LandError::Trailers)?
        .zone
        .ok_or_else(|| LandError::NoZone { commit: squash.commit.clone() })
}
