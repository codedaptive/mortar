//! zone_move_op: the op that moves a zone branch after a publish, and the zone's replay records with it.
//! No sibling unit test (S-104): every test of it runs gitio's Oid Display in the key and reason (S-101). Covered by
//! tests/after_publish_zit (zone_moves.rs): another_zone_is_replayed_onto_the_new_origin reads its zone move and
//! record write from the log, the_publishing_zone_with_no_later_joins_is_put_on_origin its kind and move, and
//! a_repeat_moves_nothing counts its records; (replay_provenance.rs)
//! two_saves_with_the_same_metadata_record_the_replay_of_the_later_one_and_publish_it reads the record ref it wrote;
//! (record_consumed.rs) the_publish_that_consumes_the_record_deletes_it reads the record it deleted, and
//! a_repeat_after_the_consuming_publish_moves_nothing counts its records; (records_deleted.rs)
//! a_replaying_publish_deletes_the_records_at_or_behind_its_source_and_writes_its_own_in_one_op reads every move
//! of the one op; its unit cases are Nagatha's RITs (owed).

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `after-publish`: the move of `zone_ref` from `old` to `new`, then the `records` moves (record_moves: the
/// deletes of the records at or behind the published source, then the records a replay writes and carries), so one
/// op moves the zone and its replay records together.
/// The key is `after-publish:<ref>:<old>:<new>:<op_id>`: it carries the op id so each attempt is its own op, as
/// squash_op's and integration_op's keys do, because a done record of an earlier move (one later undone, say) must
/// never answer this one. The reason names the ref and both ids.
pub(crate) fn zone_move_op(
    zone_ref: &str,
    old: &Oid,
    new: &Oid,
    records: Vec<RefMove>,
    op_id: String,
    actor: &str,
    at: u64,
) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("after-publish:{zone_ref}:{old}:{new}:{op_id}"),
        op_id,
        kind: "after-publish".to_string(),
        actor: actor.to_string(),
        reason: format!("after-publish moves {zone_ref} from {old} to {new}"),
        at,
        moves: std::iter::once(RefMove { refname: zone_ref.to_string(), old: Some(old.clone()), new: new.clone() })
            .chain(records)
            .collect(),
    }
}
