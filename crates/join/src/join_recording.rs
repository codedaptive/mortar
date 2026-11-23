//! join_recording: join, recording the request's base before and after on its rebase op when the caller knows the
//! base the request was made on. It takes no lock: the caller holds join-<zone> and has run recover.
//! No unit test: it runs git, applies ops and, through pause, writes real files; decide_join_tests,
//! after_replay_tests, encode_join_state_tests, fast_forward_op_tests, rebase_op_tests, base_values_tests and
//! op_applied_tests cover its decisions.
//! Integration tests are to cover, in a throwaway repository: a fast-forward join; a zone moved under a request
//! rebases and stops with the request ref moved and the zone not moved; a conflict lists the files, writes
//! mortar-join.json and moves no ref; a repeat of a joined request answers Joined and moves nothing; a missing zone
//! or request ref is refused (ZoneMissing, RequestMissing). tests/partner_base/recorded_base.rs covers the values:
//! plain_rebase_records_bases_and_survives_undo (a known base) and plain_join_without_known_base_is_not_recorded.

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::base_values::base_values;
use crate::decide_join::decide_join;
use crate::fast_forward_op::fast_forward_op;
use crate::join_decision::JoinDecision;
use crate::join_read::JoinRead;
use crate::op_applied::op_applied;
use crate::pause::pause;
use crate::rebase_op::rebase_op;
use crate::{replay, JoinAnswer, JoinRunError, RequestBase};
use gitio::Git;
use oplog::{apply_protected, apply_with_values, new_op_id, unix_now, ProtectedRef};
use std::path::Path;

/// Joins `request_ref` (a full ref name) to zone `zone`, whose ref is `refs/heads/zone/<zone>`: reads both tips and
/// their merge base, and decides. A request tip already in the zone answers Joined and moves nothing. A zone tip
/// behind the request moves the zone ref forward by one op, applied with the zone ref protected, so a move that is
/// not a fast-forward is refused. Otherwise the request's commits after the fork are replayed onto the zone tip:
/// clean, the request ref moves to the replayed tip by one op, the zone does not move, and Rebased carries the
/// request ref's old and new tips so the caller can move the request's folder, with no folder_tree (the
/// folder moves from `from`); conflicted, no ref moves and the state is written to `folder`'s git dir as
/// mortar-join.json, with no base and `before` as its base before. With `before` (the base the request was made on),
/// the rebase op records base_values(before, the zone tip with no partners) in its values; with None it records
/// none. Ops name `actor` as who moved the refs. Each op's key carries its op id, so a done record already holding
/// that key means this join moved nothing: it is refused (OpRepeated), never answered as a move.
pub(crate) fn join_recording(
    git: &Git,
    folder: &Path,
    zone: &str,
    request_ref: &str,
    actor: &str,
    before: Option<&RequestBase>,
) -> Result<JoinAnswer, JoinRunError> {
    let zone_ref = format!("refs/heads/zone/{zone}");
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let zone_tip = git
        .read_ref(&zone_ref)
        .map_err(JoinRunError::Git)?
        .ok_or_else(|| JoinRunError::ZoneMissing(zone_ref.clone()))?;
    let request_tip = git
        .read_ref(request_ref)
        .map_err(JoinRunError::Git)?
        .ok_or_else(|| JoinRunError::RequestMissing(request_ref.to_string()))?;
    let fork = git.merge_base(&request_tip, &zone_tip).map_err(JoinRunError::Git)?;
    let read = JoinRead { zone_tip, request_tip, fork };
    match decide_join(&read) {
        JoinDecision::Joined => Ok(JoinAnswer::Joined { zone_ref, request_tip: read.request_tip }),
        JoinDecision::FastForward { from, to } => {
            let op = fast_forward_op(&zone_ref, &from, &to, new_op_id(), actor, unix_now());
            let protected = [ProtectedRef { refname: zone_ref.clone(), by: "join".to_string() }];
            op_applied(apply_protected(git, &common, &op, &protected))?;
            Ok(JoinAnswer::FastForwarded { zone_ref, from, to })
        }
        JoinDecision::Replay { base, onto } => {
            let result = replay(git, &base, request_ref, &onto).map_err(JoinRunError::Replay)?;
            match after_replay(request_ref, &read.request_tip, &onto, &[], None, before, result) {
                AfterReplayStep::Rebase { from, to } => {
                    let op = rebase_op(request_ref, &from, &to, &onto, new_op_id(), actor, unix_now());
                    let after = RequestBase { base: onto, partners: vec![] };
                    let values = before.map_or_else(Vec::new, |before| base_values(before, &after));
                    op_applied(apply_with_values(git, &common, &op, &values).map(|valued| valued.outcome))?;
                    Ok(JoinAnswer::Rebased { request_ref: request_ref.to_string(), from, to, folder_tree: None })
                }
                AfterReplayStep::Pause(state) => pause(git, folder, state),
            }
        }
        JoinDecision::Unrelated => Err(JoinRunError::Unrelated { zone_ref, request_ref: request_ref.to_string() }),
    }
}
