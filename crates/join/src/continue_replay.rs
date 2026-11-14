//! continue_replay: a resolve's replay, from the resolved commit through the commits still to replay.
//! No unit test: it composes join functions (check_onto, tree_with_paths_typed through ReplayGit, resolved_commit,
//! replay_commits, after_replay, tree_refusal), so its tests are integration (rule 11; WO-join-3g order).
//! Nagatha's integration tests are to cover:
//! - the resolved tree is tree_with_paths_typed(state.tree, the caller's tree, state.files) and is the tree committed
//!   (S-50);
//! - with no remaining commits, the step rebases the request ref from the request tip to the resolved commit
//!   (WO-join-3 order, Goal 3: "Clean: one op moves the request branch");
//! - clean remaining commits chain onto the resolved commit (SPEC-join §WO-join-3: "Use it as the resolved commit
//!   and continue the replay from S-21's state");
//! - a remaining commit that conflicts pauses with a new state (WO-join-3 order, Goal 3, "Conflicted again") whose
//!   all_files are the earlier rounds' paths then the new round's (S-87, WO-join-3d);
//! - a conflicted path that is a folder in the caller's tree is refused before any commit (WO-join-3 order,
//!   "Rework ruled 2026-11-14", item 6);
//! - a missing onto is refused before any other call (SPEC-join §WO-join-1: "Before any replay, merge_tree or
//!   attr_check, join checks the `--attr-source` commit exists").

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::check_onto::check_onto;
use crate::join_state::JoinState;
use crate::replay_commits::replay_commits;
use crate::replay_git::ReplayGit;
use crate::resolved_commit::resolved_commit;
use crate::tree_refusal::tree_refusal;
use crate::JoinRunError;
use gitio::Oid;

/// Checks the state's onto exists, then builds the resolved tree: the state's conflicted tree with each conflicted
/// path taken from `caller_tree` (the folder's tree, which the caller passes), so edits outside the conflicted files
/// are not part of it (S-50). Commits the resolved tree as the resolved commit and replays the state's remaining
/// commits onto it, reading merge attributes from onto. Clean: rebase the request ref from the state's request tip
/// to the last new commit. Conflicted: pause with a new state for the commit that did not apply, the same request
/// ref, request tip and onto, and all_files the state's all_files with the new round's paths added. A failure
/// building the resolved tree is mapped by tree_refusal (a refused conflicted path, or Git); any other failure is
/// Replay.
/// Writes objects; moves no ref.
pub(crate) fn continue_replay(
    git: &impl ReplayGit,
    state: &JoinState,
    caller_tree: &Oid,
) -> Result<AfterReplayStep, JoinRunError> {
    check_onto(git, &state.onto).map_err(JoinRunError::Replay)?;
    let resolved_tree = git
        .tree_with_paths_typed(&state.tree, caller_tree, &state.files)
        .map_err(|error| tree_refusal(error, &state.files))?;
    let resolved = resolved_commit(git, state, &resolved_tree).map_err(JoinRunError::Replay)?;
    let result = replay_commits(git, &state.onto, &resolved, &state.remaining).map_err(JoinRunError::Replay)?;
    Ok(after_replay(&state.request_ref, &state.request_tip, &state.onto, &state.all_files, result))
}
