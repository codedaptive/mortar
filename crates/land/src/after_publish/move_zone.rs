//! move_zone: one zone after a publish, put on or replayed onto origin, or left in place when it conflicts or its
//! replay dropped a commit owing a record.
//! No unit test: it runs git and writes the op log (S-5); decide_zone_case_tests, consumed_record_tests,
//! record_moves_tests, clean_replay_move_tests and requests_to_rebase_tests cover its decisions. Covered by
//! tests/after_publish_zit (zone_moves.rs): every case moves its zones through it;
//! (replayed_source.rs) the publishing zone whose squash source an earlier after_publish replayed; and
//! (replay_provenance.rs) that source told apart by the replay record: two saves sharing message, author and date,
//! a re-save sharing them refused, the source pruned; and (record_consumed.rs) the record deleted by the move past
//! its source, put on origin or replayed, brought back by undo of that op, and a repeat that moves nothing; and
//! (replay_drop.rs) a zone whose replay dropped a commit owing a record left in place, then moved at its own
//! publish.

use crate::after_publish::{
    clean_replay_move, consumed_record, decide_zone_case, read_held, read_record_move, read_squash_of, record_moves,
    replay_record_ref, requests_to_rebase, zone_move_op, AfterPublishError, SquashOfRead, ZoneCase, ZoneMove,
    ZoneRequests,
};
use crate::landing::landing_applied;
use crate::squash::read_tip;
use gitio::{Git, Oid};
use join::{replay, ReplayResult};
use oplog::{apply, new_op_id, unix_now, RefMove};
use std::ops::ControlFlow;
use std::path::Path;

/// Reads the zone tip (a missing branch is Squash) and its merge base with `origin` (none: Unrelated); for the
/// publishing zone also the squash's Mortar-Squash-Of on origin's line and the zone's commit standing for it
/// (read_held): the replay of it recorded at `refs/mortar/replayed/<zone>/<source>` when the tip holds that, else the
/// commit itself when it exists and the tip holds it, and that record's value. Then, by decide_zone_case: Current
/// moves nothing. Advance moves the zone branch to origin in one op. Replay replays the zone's commits after the
/// case's base onto origin (join's replay, merge attributes read from origin): clean, one op moves the branch to the
/// replayed tip and, when the zone has a squash out whose source the replay carried (read_record_move), its replay
/// record with it; clean but the replay dropped a commit owing a record (read_record_move Unmappable), nothing moves
/// and the answer is ReplayDropped (clean_replay_move, S-120); conflicted, nothing moves and the answer names the
/// commit and files. An Advance or a clean Replay
/// of the publishing zone moves it past its squash source, so the same op deletes that source's replay record
/// (consumed_record, S-118 (1)); record_moves keeps a carried record of the same ref instead.
/// NoSquashOf and SquashOfNotInZone are refused. Only the zone branch and its replay records move; origin and what it
/// reaches are never rewritten.
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
        true => match read_squash_of(git, origin, &tip, &zone.zone)? {
            None => SquashOfRead::Missing,
            Some(commit) => read_held(git, &zone.zone, &commit, &tip)?,
        },
    };
    let case = decide_zone_case(&tip, &base, origin, &squash_of);
    let (new, carried) = match &case {
        ZoneCase::Current => return Ok(ZoneMove::Current { zone: zone.zone.clone(), tip }),
        ZoneCase::Advance => (origin.clone(), None),
        ZoneCase::Replay { base } => match replay(git, base, &zone_ref, origin).map_err(AfterPublishError::Join)? {
            ReplayResult::Clean(new) => {
                let read = read_record_move(git, &zone.zone, base, &tip, origin, &new)?;
                match clean_replay_move(&zone.zone, &tip, new, read) {
                    ControlFlow::Continue(moved) => moved,
                    ControlFlow::Break(answer) => return Ok(answer),
                }
            }
            ReplayResult::Conflicted { commit, files, .. } => {
                return Ok(ZoneMove::Conflicted { zone: zone.zone.clone(), tip, commit, files })
            }
        },
        ZoneCase::NoSquashOf => return Err(AfterPublishError::NoSquashOf { zone: zone.zone.clone() }),
        ZoneCase::SquashOfNotInZone { squash_of } => {
            return Err(AfterPublishError::SquashOfNotInZone { zone: zone.zone.clone(), squash_of: squash_of.clone() })
        }
    };
    let records = record_moves(consumed_move(&zone.zone, &case, &squash_of)?, carried);
    let op = zone_move_op(&zone_ref, &tip, &new, records, new_op_id(), actor, unix_now());
    landing_applied(apply(git, common, &op)).map_err(AfterPublishError::Land)?;
    let rebase = requests_to_rebase(&zone.requests, &new);
    Ok(ZoneMove::Moved { zone: zone.zone.clone(), old: tip, new, rebase })
}

/// The delete of the record consumed_record names, its new value the all-zero id of the record's length (the oplog's
/// delete, S-9). A zero id that does not parse is Git.
fn consumed_move(zone: &str, case: &ZoneCase, squash_of: &SquashOfRead) -> Result<Option<RefMove>, AfterPublishError> {
    let (Some(recorded), SquashOfRead::Found { commit, .. }) = (consumed_record(case, squash_of), squash_of) else {
        return Ok(None);
    };
    let zero = Oid::parse(&"0".repeat(recorded.as_str().len())).map_err(AfterPublishError::Git)?;
    Ok(Some(RefMove { refname: replay_record_ref(zone, commit.as_str()), old: Some(recorded), new: zero }))
}
