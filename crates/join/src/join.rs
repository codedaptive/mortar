//! join: joins a request ref to its zone, under the zone's join lock.
//! No unit test: it takes a real lock, runs git and writes real files; decide_join_tests, after_replay_tests,
//! encode_join_state_tests, fast_forward_op_tests, rebase_op_tests and op_applied_tests cover its decisions.
//! Nagatha's integration tests are to cover, in a throwaway repository (SPEC-join §Tests): a fast-forward join; a
//! zone moved under a request rebases and stops with the request ref moved and the zone not moved; a conflict
//! lists the files, writes mortar-join.json and moves no ref; a repeat of a joined request answers Joined and moves
//! nothing; a kill mid-join recovers and a repeat finishes once; a missing zone or request ref is refused.

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::decide_join::decide_join;
use crate::encode_join_state::encode_join_state;
use crate::fast_forward_op::fast_forward_op;
use crate::join_decision::JoinDecision;
use crate::join_read::JoinRead;
use crate::join_state::JoinState;
use crate::op_applied::op_applied;
use crate::rebase_op::rebase_op;
use crate::with_join_lock::with_join_lock;
use crate::write_join_state::write_join_state;
use crate::{replay, JoinAnswer, JoinRunError};
use gitio::Git;
use oplog::{apply, apply_protected, new_op_id, recover, unix_now, PidProbe, ProtectedRef};
use std::path::Path;

/// Joins `request_ref` (a full ref name) to zone `zone`, whose ref is `refs/heads/zone/<zone>`. Under the zone's
/// join lock (op key `join:<zone>:<request_ref>`): settles any open op (recover), reads both tips and their merge
/// base, and decides. A request tip already in the zone answers Joined and moves nothing. A zone tip behind the
/// request moves the zone ref forward by one op, applied with the zone ref protected, so a move that is not a
/// fast-forward is refused. Otherwise the request's commits after the fork are replayed onto the zone tip: clean,
/// the request ref moves to the replayed tip by one op and the zone does not move (the caller moves the request's
/// folder); conflicted, no ref moves and the state is written to `folder`'s git dir as mortar-join.json. Ops name
/// the lock holder as `<pid>@<host>`. Each op's key carries its op id, so a done record already holding that key
/// means this join moved nothing: it is refused (OpRepeated), never answered as a move.
pub fn join(
    git: &Git,
    folder: &Path,
    zone: &str,
    request_ref: &str,
    pids: &dyn PidProbe,
) -> Result<JoinAnswer, JoinRunError> {
    let zone_ref = format!("refs/heads/zone/{zone}");
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let key = format!("join:{zone}:{request_ref}");
    with_join_lock(git, &common, zone, &key, pids, |holder| {
        recover(git, &common).map_err(JoinRunError::Oplog)?;
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
        let actor = format!("{}@{}", holder.pid, holder.host);
        match decide_join(&read) {
            JoinDecision::Joined => Ok(JoinAnswer::Joined { zone_ref, request_tip: read.request_tip }),
            JoinDecision::FastForward { from, to } => {
                let op = fast_forward_op(&zone_ref, &from, &to, new_op_id(), &actor, unix_now());
                let protected = [ProtectedRef { refname: zone_ref.clone(), by: "join".to_string() }];
                op_applied(apply_protected(git, &common, &op, &protected))?;
                Ok(JoinAnswer::FastForwarded { zone_ref, from, to })
            }
            JoinDecision::Replay { base, onto } => {
                let result = replay(git, &base, request_ref, &onto).map_err(JoinRunError::Replay)?;
                match after_replay(request_ref, &read.request_tip, &onto, result) {
                    AfterReplayStep::Rebase { from, to } => {
                        let op = rebase_op(request_ref, &from, &to, &onto, new_op_id(), &actor, unix_now());
                        op_applied(apply(git, &common, &op))?;
                        Ok(JoinAnswer::Rebased { request_ref: request_ref.to_string(), from, to })
                    }
                    AfterReplayStep::Pause(state) => pause(git, folder, state),
                }
            }
            JoinDecision::Unrelated => Err(JoinRunError::Unrelated { zone_ref, request_ref: request_ref.to_string() }),
        }
    })
}

/// Writes `state` as mortar-join.json in `folder`'s git dir and answers Conflicted naming it. A conflicted path
/// that is not UTF-8 is refused (StatePath) before anything is written.
fn pause(git: &Git, folder: &Path, state: JoinState) -> Result<JoinAnswer, JoinRunError> {
    let text = encode_join_state(&state).map_err(JoinRunError::StatePath)?;
    let git_dir = git.git_dir(folder).map_err(JoinRunError::Git)?;
    let path = write_join_state(&git_dir, &text)?;
    Ok(JoinAnswer::Conflicted { commit: state.commit, files: state.files, state: path })
}
