//! after_replay: the step after a replay, from its result. after_replay_tests covers both arms.

use crate::after_replay_step::AfterReplayStep;
use crate::join_state::JoinState;
use crate::ReplayResult;
use gitio::Oid;

/// A clean replay rebases the request ref from `request_tip` to the new tip. A conflicted one pauses with
/// a JoinState carrying the request ref, its tip, `onto` and every field of the conflict.
pub(crate) fn after_replay(request_ref: &str, request_tip: &Oid, onto: &Oid, result: ReplayResult) -> AfterReplayStep {
    match result {
        ReplayResult::Clean(new) => AfterReplayStep::Rebase { from: request_tip.clone(), to: new },
        ReplayResult::Conflicted { commit, replayed_tip, remaining, tree, files } => AfterReplayStep::Pause(JoinState {
            request_ref: request_ref.to_string(),
            request_tip: request_tip.clone(),
            onto: onto.clone(),
            replayed_tip,
            commit,
            tree,
            remaining,
            files,
        }),
    }
}
