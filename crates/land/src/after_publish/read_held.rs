//! read_held: the zone commit standing for the publishing zone's squash source.
//! No unit test: it only reads git (S-5); choose_held_tests covers every branch of the choice. Covered by
//! tests/after_publish_zit (zone_moves.rs): the_publishing_zone_with_no_later_joins_is_put_on_origin and
//! the_publishing_zone_with_a_later_join_is_replayed_onto_origin (the source in the zone); (replayed_source.rs)
//! the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin and
//! a_request_joined_after_the_replay_is_replayed_onto_origin (the recorded replay); (replay_provenance.rs)
//! two_saves_with_the_same_metadata_record_the_replay_of_the_later_one_and_publish_it,
//! a_re_save_with_the_same_metadata_after_the_replay_is_refused_not_dropped and
//! the_publishing_zone_moves_when_its_squash_source_was_pruned.

use crate::after_publish::{choose_held, replay_record_ref, AfterPublishError};
use gitio::{Git, Oid};

/// Reads the record `replay_record_ref(zone, source)` and, when there is one, whether `tip` holds its commit; then
/// whether `source` exists and, when it does, whether `tip` holds it. The existence read guards the ancestry read,
/// which git refuses for a missing object. Then choose_held. A failed git run is Git.
pub(crate) fn read_held(git: &Git, zone: &str, source: &Oid, tip: &Oid) -> Result<Option<Oid>, AfterPublishError> {
    let held = |commit: &Oid| git.is_ancestor(commit, tip).map_err(AfterPublishError::Git);
    Ok(choose_held(
        source,
        git.read_ref(&replay_record_ref(zone, source.as_str()))
            .map_err(AfterPublishError::Git)?
            .map(|commit| held(&commit).map(|is_held| (commit, is_held)))
            .transpose()?,
        git.rev_parse(source.as_str()).map_err(AfterPublishError::Git)?.map(|_| held(source)).transpose()?,
    ))
}
