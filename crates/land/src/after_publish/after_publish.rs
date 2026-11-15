//! after_publish: every zone moved onto the origin a publish left (SPEC-land 05-after-publish.md, S-45, S-47).
//! No unit test (S-5): it runs git and writes the op log through move_zone; decide_zone_case_tests,
//! squash_of_in_tests, requests_to_rebase_tests, choose_held_tests and replayed_position_tests cover its
//! decisions. Covered by tests/after_publish_zit:
//! - zone_moves.rs: another_zone_is_replayed_onto_the_new_origin,
//!   the_publishing_zone_with_no_later_joins_is_put_on_origin,
//!   the_publishing_zone_with_a_later_join_is_replayed_onto_origin,
//!   a_conflicting_zone_is_left_in_place_and_listed_with_its_files and a_repeat_moves_nothing.
//! - merge_landing.rs: the_publishing_zone_landed_as_a_merge_is_found_through_the_second_parent.
//! - replayed_source.rs: the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin and
//!   a_request_joined_after_the_replay_is_replayed_onto_origin.
//! - replay_provenance.rs: two_saves_with_the_same_metadata_record_the_replay_of_the_later_one_and_publish_it,
//!   a_re_save_with_the_same_metadata_after_the_replay_is_refused_not_dropped and
//!   the_publishing_zone_moves_when_its_squash_source_was_pruned.

use crate::after_publish::{move_zone, AfterPublishError, ZoneMove, ZoneRequests};
use crate::squash::{read_tip, LandRefs};
use gitio::Git;

/// Reads the origin tip `refs/heads/<origin_branch>` once, then moves the `published` zone (the one whose squash
/// was just published) and each of `others` in order onto it, one ZoneMove each, the published zone first. The
/// published zone with no join since its squash (or since the recorded replay of its squash's source) is put on
/// origin; one with later joins has those replayed onto origin. Another zone has its own commits replayed onto
/// origin, or is put on it when it has none. A zone already on origin answers Current, so a repeat is safe and moves
/// nothing. A conflicted zone is left in place, listed with its files, and the others still move; an error stops the
/// call, the zones before it already moved.
/// It takes no lock: the caller holds `land` and has run recover (S-16). The caller hands in each zone's open
/// requests (S-45) and moves the request and zone folders itself (folders move_folder, S-47).
pub fn after_publish(
    git: &Git,
    refs: &LandRefs,
    published: &ZoneRequests,
    others: &[ZoneRequests],
    actor: &str,
) -> Result<Vec<ZoneMove>, AfterPublishError> {
    let common = git.common_dir().map_err(AfterPublishError::Git)?;
    let origin = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(AfterPublishError::Squash)?;
    std::iter::once((published, true))
        .chain(others.iter().map(|zone| (zone, false)))
        .map(|(zone, publishing)| move_zone(git, &common, &origin, zone, publishing, actor))
        .collect()
}
