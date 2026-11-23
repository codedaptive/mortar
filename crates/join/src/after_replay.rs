//! after_replay: the step after a replay, from its result. after_replay_tests covers both arms, a pause carrying
//! earlier rounds' paths included.

use crate::accumulate_paths::accumulate_paths;
use crate::after_replay_step::AfterReplayStep;
use crate::join_state::JoinState;
use crate::request_base::RequestBase;
use crate::ReplayResult;
use gitio::Oid;
use std::path::PathBuf;

/// A clean replay rebases the request ref from `request_tip` to the new tip. A conflicted one pauses with
/// a JoinState carrying the request ref, its tip, `onto` and every field of the conflict, with no files written
/// yet, no rebased tip and no folder tree. Its `all_files` are `earlier` (the conflicted paths of this join's earlier
/// rounds; empty for join's own replay) followed by this round's new paths (accumulate_paths), and its `base` a copy
/// of `base` (the request's base and partners after this join; None for a plain join).
pub(crate) fn after_replay(
    request_ref: &str,
    request_tip: &Oid,
    onto: &Oid,
    earlier: &[PathBuf],
    base: Option<&RequestBase>,
    result: ReplayResult,
) -> AfterReplayStep {
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
            all_files: accumulate_paths(earlier, &files),
            files,
            files_written: false,
            rebased_to: None,
            folder_tree: None,
            base: base.cloned(),
        }),
    }
}
