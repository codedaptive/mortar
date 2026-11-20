//! move_zone: one zone after a publish, put on or replayed onto origin, or left in place when it conflicts, with its
//! replay records written, carried and deleted in the op that moves it.
//! No unit test: it runs git and writes the op log; decide_zone_case_tests, published_records_tests,
//! replay_records_tests, record_moves_tests, records_op_tests and requests_to_rebase_tests cover its decisions.
//! Covered by tests/after_publish_zit (zone_moves.rs): every case moves its zones through it, and
//! another_zone_is_replayed_onto_the_new_origin reads the record a replay writes; (replayed_source.rs) the
//! publishing zone whose squash source an earlier after_publish replayed; (replay_provenance.rs) that source told
//! apart by the replay record; (record_consumed.rs) the record deleted by the move past its source, put on origin or
//! replayed, and a repeat that moves nothing; (replay_drop.rs) a zone whose replay drops a commit moved, the dropped
//! commit recorded; (restored_zone_ref.rs) a record rewritten by the next replay; (squash_ref_replaced.rs) every
//! replayed commit recorded and a record carried; (records_deleted.rs) the records at or behind the source deleted
//! in the zone's op, and only the source's own record in a records-only op when Current from Superseded;
//! (older_squash.rs) an older squash left Current with no op, and one not behind refused.

use crate::after_publish::{
    decide_zone_case, published_records, read_publishing, read_zone_records, record_moves, records_op, replay_records,
    requests_to_rebase, zone_move_op, AfterPublishError, SquashOfRead, ZoneCase, ZoneMove, ZoneRequests,
};
use crate::landing::landing_applied;
use crate::squash::read_tip;
use gitio::{Git, Oid};
use join::{replay_mapped, MappedCommit, ReplayResult};
use oplog::{apply, new_op_id, unix_now, Op, RefMove};
use std::path::Path;

/// Reads the zone tip (a missing branch is Squash) and its merge base with `origin` (none: Unrelated); for the
/// publishing zone also what it holds for the published squash's source (read_publishing). Then, by
/// decide_zone_case: NoSquashOf and SquashOfNotInZone are refused, moving nothing. Replay replays the zone's commits
/// after the case's base onto origin commit by commit (join's replay_mapped, merge attributes read from origin);
/// conflicted, nothing moves and the answer names the commit and files. Advance and a clean Replay move the zone
/// branch in one op that also moves its replay records (record_moves): the publishing zone's records at or behind
/// the published source deleted (published_records), and a clean replay's records written for every
/// replayed commit and carried forward (replay_records). Current moves no branch; the publishing zone's
/// deletes, when there are any, go in one records-only op (records_op), and otherwise no op is written. Another
/// zone deletes no record. Origin and what it reaches are never rewritten.
pub(crate) fn move_zone(
    git: &Git,
    common: &Path,
    origin: &Oid,
    zone: &ZoneRequests,
    publishing: bool,
    actor: &str,
) -> Result<ZoneMove, AfterPublishError> {
    let zone_ref = format!("refs/heads/zone/{}", zone.zone);
    let tip = read_tip(git, &zone_ref).map_err(AfterPublishError::Squash)?;
    let base = git.merge_base(origin, &tip).map_err(AfterPublishError::Git)?.ok_or_else(|| {
        AfterPublishError::Unrelated { zone: zone.zone.clone(), origin: origin.clone(), tip: tip.clone() }
    })?;
    let squash_of = match publishing {
        false => SquashOfRead::NotPublishing,
        true => read_publishing(git, &zone.zone, origin, &tip, &base)?,
    };
    let (new, mapped) = match decide_zone_case(&tip, &base, origin, &squash_of) {
        ZoneCase::NoSquashOf => return Err(AfterPublishError::NoSquashOf { zone: zone.zone.clone() }),
        ZoneCase::SquashOfNotInZone { squash_of } => {
            return Err(AfterPublishError::SquashOfNotInZone { zone: zone.zone.clone(), squash_of })
        }
        ZoneCase::Current => (None, vec![]),
        ZoneCase::Advance => (Some(origin.clone()), vec![]),
        ZoneCase::Replay { base } => {
            let replay = replay_mapped(git, &base, &zone_ref, origin).map_err(AfterPublishError::Join)?;
            match replay.result {
                ReplayResult::Clean(new) => (Some(new), replay.mapped),
                ReplayResult::Conflicted { commit, files, .. } => {
                    return Ok(ZoneMove::Conflicted { zone: zone.zone.clone(), tip, commit, files })
                }
            }
        }
    };
    let (deletes, writes) = record_changes(git, &zone.zone, &squash_of, &mapped)?;
    let Some(new) = new else {
        if !deletes.is_empty() {
            applied(git, common, records_op(&zone_ref, deletes, new_op_id(), actor, unix_now()))?;
        }
        return Ok(ZoneMove::Current { zone: zone.zone.clone(), tip });
    };
    let moves = record_moves(deletes, writes);
    applied(git, common, zone_move_op(&zone_ref, &tip, &new, moves, new_op_id(), actor, unix_now()))?;
    let rebase = requests_to_rebase(&zone.requests, &new);
    Ok(ZoneMove::Moved { zone: zone.zone.clone(), old: tip, new, rebase })
}

/// The publishing zone's deletes (published_records of the source its squash named, over its records read against
/// the commit it holds for that source) and a clean replay's writes (replay_records over `mapped`). The records are
/// read only when either can be non-empty: for the publishing zone, or for another zone's replay. A zero id that
/// does not parse, or a failed git run, is Git.
fn record_changes(
    git: &Git,
    zone: &str,
    squash_of: &SquashOfRead,
    mapped: &[MappedCommit],
) -> Result<(Vec<RefMove>, Vec<RefMove>), AfterPublishError> {
    let (source, held) = match squash_of {
        SquashOfRead::Found { commit, held } => (Some(commit), held.as_ref()),
        SquashOfRead::Superseded { commit } => (Some(commit), None),
        SquashOfRead::NotPublishing | SquashOfRead::Missing => (None, None),
    };
    if source.is_none() && mapped.is_empty() {
        return Ok((vec![], vec![]));
    }
    let records = read_zone_records(git, zone, held)?;
    let deletes = match source {
        Some(source) => published_records(zone, source, &records).map_err(AfterPublishError::Git)?,
        None => vec![],
    };
    Ok((deletes, replay_records(zone, mapped, &records)))
}

/// Applies `op` through the op log; a refused or failed op is Land.
fn applied(git: &Git, common: &Path, op: Op) -> Result<(), AfterPublishError> {
    landing_applied(apply(git, common, &op)).map_err(AfterPublishError::Land)
}
