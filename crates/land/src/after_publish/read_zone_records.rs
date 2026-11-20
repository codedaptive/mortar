//! read_zone_records: a zone's replay records, each with whether it is at or behind a held commit.
//! No unit test: it only reads git; replay_records_tests and published_records_tests cover what is decided
//! from its answer. Covered by tests/after_publish_zit: (records_deleted.rs)
//! a_replaying_publish_deletes_the_records_at_or_behind_its_source_and_writes_its_own_in_one_op (a record behind
//! the held commit) and a_current_publish_deletes_only_the_stale_record_of_its_source_in_a_records_only_op (no held
//! commit); (replay_drop.rs)
//! a_zone_whose_replay_drops_a_commit_moves_records_it_as_its_onto_and_its_publish_deletes_both_records;
//! (squash_ref_replaced.rs) an_older_squash_published_after_a_newer_one_replaced_the_squash_ref_moves_the_zone (a
//! record carried, one not behind); (restored_zone_ref.rs)
//! a_zone_ref_restored_behind_its_record_has_the_record_rewritten_by_the_next_replay (a record's old value read).

use crate::after_publish::{replay_record_ref, AfterPublishError, ZoneRecord};
use gitio::{Git, Oid};

/// Lists `refs/mortar/replayed/<zone>/` (replay_record_ref with an empty source), in git's order. Each record's
/// `behind` is whether its value is `held` or an ancestor of it; false for every record when `held` is None, and
/// then no ancestry is read. A failed git run is Git.
pub(crate) fn read_zone_records(
    git: &Git,
    zone: &str,
    held: Option<&Oid>,
) -> Result<Vec<ZoneRecord>, AfterPublishError> {
    let listed = git.list_refs(&replay_record_ref(zone, "")).map_err(AfterPublishError::Git)?;
    listed
        .into_iter()
        .map(|(commit, refname)| {
            let behind = held.map(|held| git.is_ancestor(&commit, held)).transpose().map_err(AfterPublishError::Git)?;
            Ok(ZoneRecord { refname, commit, behind: behind.unwrap_or(false) })
        })
        .collect()
}
