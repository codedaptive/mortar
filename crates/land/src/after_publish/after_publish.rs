//! after_publish: every zone moved onto the origin a publish left (SPEC-land 05-after-publish.md, S-45, S-47, S-121).
//! No unit test (S-5): it runs git and writes the op log through move_zone; decide_zone_case_tests,
//! squash_of_in_tests, requests_to_rebase_tests, choose_held_tests, published_records_tests, replay_records_tests,
//! record_moves_tests and records_op_tests cover its decisions. Covered by tests/after_publish_zit:
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
//! - record_consumed.rs: the_publish_that_consumes_the_record_deletes_it, a_replayed_publish_deletes_the_record_too
//!   and a_repeat_after_the_consuming_publish_moves_nothing.
//! - replay_drop.rs:
//!   a_zone_whose_replay_drops_a_commit_moves_records_it_as_its_onto_and_its_publish_deletes_both_records.
//! - restored_zone_ref.rs: a_zone_ref_restored_behind_its_record_has_the_record_rewritten_by_the_next_replay.
//! - squash_ref_replaced.rs: an_older_squash_published_after_a_newer_one_replaced_the_squash_ref_moves_the_zone.
//! - source_not_read.rs: a_held_record_moves_the_zone_without_reading_the_source.
//! - records_deleted.rs: a_replaying_publish_deletes_the_records_at_or_behind_its_source_and_writes_its_own_in_one_op
//!   and a_current_publish_deletes_only_the_stale_record_of_its_source_in_a_records_only_op.
//! - older_squash.rs: an_older_squash_published_after_the_newer_one_leaves_the_zone_current_and_writes_no_op and
//!   a_squash_whose_source_is_not_behind_the_published_one_is_refused.

use crate::after_publish::{move_zone, AfterPublishError, ZoneMove, ZoneRequests};
use crate::squash::{read_tip, LandRefs};
use gitio::Git;

/// Reads the origin tip `refs/heads/<origin_branch>` once, then moves the `published` zone (the one whose squash
/// was just published) and each of `others` in order onto it, one ZoneMove each, the published zone first. The
/// published zone with no join since its squash (or since the recorded replay of its squash's source) is put on
/// origin; one with later joins has those replayed onto origin; either move deletes, in the same op, every replay
/// record of the zone at or behind its squash's source (S-121). Another zone has its own commits replayed onto
/// origin, or is put on it when it has none. Every clean replay records, in the op that moves its zone, the zone
/// commit standing for each commit it replayed. A zone already on origin answers Current, so a repeat is safe and
/// moves nothing. The published zone also answers Current, its branch unmoved, when its squash's source is at or
/// behind one already published; a records-only op then deletes the record keyed by that source and, only when the
/// zone holds that source (itself or through a record), the records at or behind it (S-129), and with nothing to
/// delete no op is written. A conflicted zone is left in place and listed with its files; the others still move; an
/// error stops the call, the zones before it already moved.
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
