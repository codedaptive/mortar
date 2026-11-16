//! read_held: what the publishing zone holds for its squash source, and the source's replay record.
//! No unit test: it only reads git (S-5); choose_held_tests covers every branch of the choice. Covered by
//! tests/after_publish_zit (zone_moves.rs): the_publishing_zone_with_no_later_joins_is_put_on_origin and
//! the_publishing_zone_with_a_later_join_is_replayed_onto_origin (the source in the zone); (replayed_source.rs)
//! the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin and
//! a_request_joined_after_the_replay_is_replayed_onto_origin (the recorded replay); (replay_provenance.rs)
//! two_saves_with_the_same_metadata_record_the_replay_of_the_later_one_and_publish_it,
//! a_re_save_with_the_same_metadata_after_the_replay_is_refused_not_dropped (a record not held, the source read)
//! and the_publishing_zone_moves_when_its_squash_source_was_pruned; (source_not_read.rs)
//! a_held_record_moves_the_zone_without_reading_the_source (no git run names the source); (restored_zone_ref.rs)
//! a_zone_ref_restored_behind_its_record_has_the_record_rewritten_by_the_next_replay (the rewritten record held).

use crate::after_publish::{choose_held, replay_record_ref, AfterPublishError, SquashOfRead};
use gitio::{Git, Oid};

/// Reads the record `replay_record_ref(zone, source)` and, when there is one, whether `tip` holds its commit. A held
/// record is the answer and the source itself is never read (rule R, S-121). Only without a held record: whether
/// `source` exists and, when it does, whether `tip` holds it; the existence read guards the ancestry read, which git
/// refuses for a missing object. Answers Found with `source` and choose_held's pick. A failed git run is Git.
pub(crate) fn read_held(git: &Git, zone: &str, source: &Oid, tip: &Oid) -> Result<SquashOfRead, AfterPublishError> {
    let held = |commit: &Oid| git.is_ancestor(commit, tip).map_err(AfterPublishError::Git);
    let record = git.read_ref(&replay_record_ref(zone, source.as_str())).map_err(AfterPublishError::Git)?;
    let recorded = record.as_ref().map(|commit| held(commit).map(|is_held| (commit.clone(), is_held))).transpose()?;
    let source_held = match recorded {
        Some((_, true)) => None,
        _ => git.rev_parse(source.as_str()).map_err(AfterPublishError::Git)?.map(|_| held(source)).transpose()?,
    };
    Ok(SquashOfRead::Found { commit: source.clone(), held: choose_held(source, recorded, source_held) })
}
