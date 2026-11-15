//! move_zone: one zone after a publish, put on or replayed onto origin, or left in place when it conflicts.
//! No unit test: it runs git and writes the op log (S-5); decide_zone_case_tests and requests_to_rebase_tests cover
//! its decisions. Covered by tests/after_publish_zit (zone_moves.rs): every case moves its zones through it; and
//! (replayed_source.rs) the publishing zone whose squash source an earlier after_publish replayed.

use crate::after_publish::{
    decide_zone_case, read_replayed_counterpart, read_squash_of, requests_to_rebase, zone_move_op, AfterPublishError,
    SquashOfRead, ZoneCase, ZoneMove, ZoneRequests,
};
use crate::landing::landing_applied;
use crate::squash::read_tip;
use gitio::{Git, Oid};
use join::{replay, ReplayResult};
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// Reads the zone tip (a missing branch is Squash) and its merge base with `origin` (none: Unrelated); for the
/// publishing zone also the squash's Mortar-Squash-Of on origin's line and the zone's commit standing for it: the
/// commit itself when the tip holds it, else its replayed counterpart between the base and the tip, which an earlier
/// after_publish made when it replayed this zone onto another zone's origin. Then, by
/// decide_zone_case: Current moves nothing. Advance moves the zone branch to origin in one op. Replay replays the
/// zone's commits after the case's base onto origin (join's replay, merge attributes read from origin): clean, one
/// op moves the branch to the replayed tip; conflicted, nothing moves and the answer names the commit and files.
/// NoSquashOf and SquashOfNotInZone are refused. Only the zone branch moves; origin and what it reaches are never
/// rewritten.
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
            Some(commit) => {
                let held = match git.is_ancestor(&commit, &tip).map_err(AfterPublishError::Git)? {
                    true => Some(commit.clone()),
                    false => read_replayed_counterpart(git, &commit, &base, &tip)?,
                };
                SquashOfRead::Found { commit, held }
            }
        },
    };
    let new = match decide_zone_case(&tip, &base, origin, &squash_of) {
        ZoneCase::Current => return Ok(ZoneMove::Current { zone: zone.zone.clone(), tip }),
        ZoneCase::Advance => origin.clone(),
        ZoneCase::Replay { base } => match replay(git, &base, &zone_ref, origin).map_err(AfterPublishError::Join)? {
            ReplayResult::Clean(new) => new,
            ReplayResult::Conflicted { commit, files, .. } => {
                return Ok(ZoneMove::Conflicted { zone: zone.zone.clone(), tip, commit, files })
            }
        },
        ZoneCase::NoSquashOf => return Err(AfterPublishError::NoSquashOf { zone: zone.zone.clone() }),
        ZoneCase::SquashOfNotInZone { squash_of } => {
            return Err(AfterPublishError::SquashOfNotInZone { zone: zone.zone.clone(), squash_of })
        }
    };
    let op = zone_move_op(&zone_ref, &tip, &new, new_op_id(), actor, unix_now());
    landing_applied(apply(git, common, &op)).map_err(AfterPublishError::Land)?;
    let rebase = requests_to_rebase(&zone.requests, &new);
    Ok(ZoneMove::Moved { zone: zone.zone.clone(), old: tip, new, rebase })
}
