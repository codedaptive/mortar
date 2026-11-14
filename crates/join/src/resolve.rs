//! resolve: continues a join that stopped on a conflict. It takes no lock: the caller holds join-<zone> and has run
//! recover (S-47, S-16).
//! No unit test: it runs git, applies ops and reads, writes and removes real files (S-5); decide_resolve_tests covers
//! its decisions, rebase_op_tests and op_applied_tests the op; continue_replay's header lists the continued replay's
//! integration cases.
//! Nagatha's integration tests are to cover, in a throwaway repository:
//! - with no state, resolve refuses (WO-join-3 Goal 1: "No state: refuse, saying there is nothing to resolve");
//! - a first run writes the conflicted tree's versions of the conflicted paths into the folder, markers included,
//!   and answers the files (WO-join-3 Goal 2); a first run repeated rewrites the same files (WO-join-3 Goal 4);
//! - a conflicted path the conflicted tree lacks (rename/rename) is skipped by the first run and listed in the answer
//!   (S-50);
//! - a next run refuses while a marker remains, naming each file and line (WO-join-3 Goal 3);
//! - a next run with the markers gone commits the conflicted tree with the conflicted paths taken from the caller's
//!   tree as the resolved commit, edits outside them left out (S-50) and, clean, moves the request branch by one op,
//!   removes mortar-join.json and answers Rebased (WO-join-3 Goal 3; S-20): "a conflict lists files, writes markers,
//!   resolves and joins" (SPEC-join §Tests);
//! - conflicted again, it writes a new state for the new commit, moves no ref and lists the files (WO-join-3 Goal 3,
//!   "Conflicted again");
//! - a next run repeated after a clean finish finds no state (WO-join-3 Goal 4); "a kill mid-join recovers and a
//!   repeat finishes once" (SPEC-join §Tests): a kill after the op, then a repeat, removes the state and moves
//!   nothing more;
//! - a kill after rebased_to is saved and before the op, then a repeat, applies the op and answers Rebased (the
//!   WO-join-3 order, "Rework ruled 2026-11-14", item 4);
//! - a request branch moved since the pause is refused as StateStale, mortar-join.json kept (the WO-join-3 order,
//!   "Rework ruled 2026-11-14", item 4);
//! - a conflicted path that became a folder in the caller's tree is refused as ConflictedPathIsFolder naming it, and
//!   nothing is committed (the WO-join-3 order, "Rework ruled 2026-11-14", item 6).

use crate::after_replay_step::AfterReplayStep;
use crate::continue_replay::continue_replay;
use crate::decide_resolve::decide_resolve;
use crate::join_state::JoinState;
use crate::op_applied::op_applied;
use crate::pause::pause;
use crate::read_join_state::read_join_state;
use crate::read_marked_files::read_marked_files;
use crate::rebase_op::rebase_op;
use crate::remove_join_state::remove_join_state;
use crate::resolve_step::ResolveStep;
use crate::save_join_state::save_join_state;
use crate::write_conflicted_files::write_conflicted_files;
use crate::{JoinAnswer, JoinRunError};
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// Resolves the join paused in `folder` (its state is mortar-join.json in the folder's git dir), taking `tree`, the
/// folder's tree, from the caller. A first run writes the conflicted files the conflicted tree has, markers included,
/// saves that it did and answers FilesWritten, listing the ones it lacks as skipped. A next run refuses while any of
/// them holds a marker line (StillMarked); otherwise it commits the conflicted tree with the conflicted paths taken
/// from `tree` as the resolved commit and continues the replay: conflicted again, it pauses with a new state and
/// answers Conflicted; clean, it saves the new tip in the state, moves the request ref there by one op naming
/// `actor`, removes the state and answers Rebased. A repeat finds where that stopped and finishes it. No state is
/// NothingToResolve; a request ref moved since the pause is StateStale, and the state is kept.
pub fn resolve(git: &Git, folder: &Path, tree: &Oid, actor: &str) -> Result<JoinAnswer, JoinRunError> {
    let git_dir = git.git_dir(folder).map_err(JoinRunError::Git)?;
    let path = git_dir.join("mortar-join.json");
    let state = read_join_state(&path)?;
    let request_now = match &state {
        Some(state) => git.read_ref(&state.request_ref).map_err(JoinRunError::Git)?,
        None => None,
    };
    let files = state.as_ref().map_or(&[][..], |state| state.files.as_slice());
    let step = decide_resolve(state.as_ref(), request_now.as_ref(), || read_marked_files(folder, files), tree)?;
    match (step, state) {
        (ResolveStep::NothingToResolve, _) | (_, None) => Err(JoinRunError::NothingToResolve { state: path }),
        (ResolveStep::Stale { expected, actual }, Some(state)) => {
            Err(JoinRunError::StateStale { request_ref: state.request_ref, expected, actual, state: path })
        }
        (ResolveStep::StillMarked(files), Some(_)) => Err(JoinRunError::StillMarked(files)),
        (ResolveStep::WriteFiles { tree, files }, Some(mut state)) => {
            let (files, skipped) = write_conflicted_files(git, folder, &tree, files)?;
            state.files_written = true;
            save_join_state(&git_dir, &state)?;
            Ok(JoinAnswer::FilesWritten { commit: state.commit, files, skipped })
        }
        (ResolveStep::Continue { tree }, Some(mut state)) => {
            match continue_replay(git, &state, &tree)? {
                AfterReplayStep::Rebase { from, to } => {
                    state.rebased_to = Some(to.clone());
                    save_join_state(&git_dir, &state)?;
                    finish(git, &path, &state, from, to, actor)
                }
                AfterReplayStep::Pause(next) => pause(git, folder, next),
            }
        }
        (ResolveStep::Rebase { from, to }, Some(state)) => finish(git, &path, &state, from, to, actor),
        (ResolveStep::Finish { from, to }, Some(state)) => {
            remove_join_state(&path)?;
            Ok(JoinAnswer::Rebased { request_ref: state.request_ref, from, to })
        }
    }
}

/// Moves the request ref from `from` to `to` by one op, as join's rebase does, then removes the state at `path`.
fn finish(
    git: &Git,
    path: &Path,
    state: &JoinState,
    from: Oid,
    to: Oid,
    actor: &str,
) -> Result<JoinAnswer, JoinRunError> {
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let op = rebase_op(&state.request_ref, &from, &to, &state.onto, new_op_id(), actor, unix_now());
    op_applied(apply(git, &common, &op))?;
    remove_join_state(path)?;
    Ok(JoinAnswer::Rebased { request_ref: state.request_ref.clone(), from, to })
}
