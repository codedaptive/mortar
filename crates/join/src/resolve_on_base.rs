//! resolve_on_base: continues a join that stopped on a conflict, answering with Rebased the base the request now
//! stands on. It takes no lock: the caller holds join-<zone> and has run recover. resolve is it without the base.
//! No unit test: it runs git, applies ops and reads, writes and removes real files; decide_resolve_tests covers
//! its decisions, rebase_op_tests and op_applied_tests the op; continue_replay's header lists the continued replay's
//! integration cases.
//! Integration tests are to cover, in a throwaway repository:
//! - with no state, resolve refuses, saying there is nothing to resolve;
//! - a first run writes the conflicted tree's versions of the conflicted paths into the folder, markers included,
//!   and answers the files; a first run repeated rewrites the same files;
//! - a conflicted path the conflicted tree lacks (rename/rename) is skipped by the first run and listed in the answer;
//! - a first run lists a conflicted gitlink (submodule) apart, with its next step, and writes no file for it;
//! - a next run takes a conflicted gitlink from the caller's tree through tree_with_paths_typed, with no marker check
//!   on it;
//! - a next run refuses while a marker remains, naming each file and line;
//! - a next run with the markers gone commits the conflicted tree with the conflicted paths taken from the caller's
//!   tree as the resolved commit, edits outside them left out and, clean, moves the request branch by one op,
//!   removes mortar-join.json and answers Rebased: a conflict lists files, writes markers, resolves and joins;
//! - conflicted again, it writes a new state for the new commit, moves no ref and lists the files;
//! - a next run repeated after a clean finish finds no state; a kill mid-join recovers and a repeat finishes once:
//!   a kill after the op, then a repeat, removes the state and moves nothing more;
//! - a kill after rebased_to is saved and before the op, then a repeat, applies the op and answers Rebased;
//! - a request branch moved since the pause is refused as StateStale, mortar-join.json kept;
//! - a conflicted path that became a folder in the caller's tree is refused as ConflictedPathIsFolder naming it, and
//!   join moves no ref;
//! - a resolve over two rounds answers Rebased with folder_tree = the old request tip's tree with every round's
//!   paths taken from the caller's tree;
//! - a kill after the final op and before the state's removal, then a repeat, answers the same Rebased with the same
//!   folder_tree, read from the state;
//! - an earlier round's path that became a folder is refused as ConflictedPathIsFolder naming it, the state kept.
//!
//! tests/partner_base moved_partner_replay_conflict_resolves_with_new_base covers the base: a join paused on a new
//! partner merge answers no base with FilesWritten, and that merge and its partners with Rebased.
//! tests/partner_base/recorded_base.rs partner_resolve_records_bases_and_survives_undo and
//! plain_resolve_records_bases_and_survives_undo cover the answered base after a remerge and after a plain replay
//! (the zone tip replayed onto), and the base before and after the final op records.
//!
//! The final op records folder_tree in its values under "folder_tree" and the state mortar-join.json held before
//! the final run under "join_state", which restore_join_state reads back, then, for a state join_on_base paused,
//! the base before and after under "base_before" and "base_after", which recorded_base reads back; while the state
//! exists a repeat answers from it; once it is removed resolve answers NothingToResolve and the cli's pending-move
//! detector reads folder_tree from the op. real_join.rs a_completed_resolve_records_folder_tree_on_its_final_op
//! covers the op's values.

use crate::after_replay_step::AfterReplayStep;
use crate::continue_replay::continue_replay;
use crate::decide_resolve::decide_resolve;
use crate::finish_resolve::finish_resolve;
use crate::join_state_path::join_state_path;
use crate::pause::pause;
use crate::read_join_state::read_join_state;
use crate::read_marked_files::read_marked_files;
use crate::remove_join_state::remove_join_state;
use crate::resolve_step::ResolveStep;
use crate::resolved_folder_tree::resolved_folder_tree;
use crate::save_join_state::save_join_state;
use crate::state_base_after::state_base_after;
use crate::write_conflicted_files::write_conflicted_files;
use crate::{JoinAnswer, JoinRunError, RequestBase};
use gitio::{Git, Oid};
use std::path::Path;

/// Resolves the join paused in `folder` (its state is mortar-join.json in the folder's git dir), taking `tree`, the
/// folder's tree, from the caller. A first run writes the conflicted files the conflicted tree has, markers included,
/// saves that it did and answers FilesWritten, listing the ones it lacks as skipped and the gitlinks (submodules)
/// apart. A next run refuses while any file holds a marker line (StillMarked), a folder (no file text)
/// having none; otherwise it commits the conflicted tree with the conflicted paths taken from `tree` as the resolved
/// commit and continues the replay: conflicted again, it pauses with a new state and answers Conflicted; clean, it
/// builds the folder tree (resolved_folder_tree), saves it with the new tip in the state, moves the request ref there
/// by one op naming `actor` that records it, the state it resolved from and, for a join_on_base pause, the base
/// before and after, removes the state and answers Rebased
/// carrying that folder_tree, the tree the caller moves the folder from. A repeat finds where
/// that stopped and finishes it with the state's folder_tree; a repeat that must still move the ref and finds none
/// is StateCorrupt, the state kept. No state is NothingToResolve; a request ref moved since the pause is StateStale,
/// and the state is kept. Beside Rebased it answers the base the request now stands on (state_base_after): the
/// state's base when the join kept one (the new merge and its partners, or the zone tip with no partners for a
/// join_on_base replay onto the unchanged zone), else the zone tip replayed onto with no partners; beside every
/// other answer, None.
pub fn resolve_on_base(
    git: &Git,
    folder: &Path,
    tree: &Oid,
    actor: &str,
) -> Result<(JoinAnswer, Option<RequestBase>), JoinRunError> {
    let git_dir = git.git_dir(folder).map_err(JoinRunError::Git)?;
    let path = join_state_path(&git_dir);
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
            let (files, skipped, gitlinks) = write_conflicted_files(git, folder, &tree, files)?;
            state.files_written = true;
            save_join_state(&git_dir, &state)?;
            Ok((JoinAnswer::FilesWritten { commit: state.commit, files, skipped, gitlinks }, None))
        }
        (ResolveStep::Continue { tree }, Some(mut state)) => {
            match continue_replay(git, &state, &tree)? {
                AfterReplayStep::Rebase { from, to } => {
                    let folder_tree = resolved_folder_tree(git, &state, &tree)?;
                    state.rebased_to = Some(to.clone());
                    state.folder_tree = Some(folder_tree.clone());
                    save_join_state(&git_dir, &state)?;
                    finish_resolve(git, &path, &state, from, to, Some(folder_tree), actor)
                        .map(|answer| (answer, Some(state_base_after(&state))))
                }
                AfterReplayStep::Pause(next) => pause(git, folder, next).map(|answer| (answer, None)),
            }
        }
        (ResolveStep::Rebase { from, to, folder_tree }, Some(state)) => {
            finish_resolve(git, &path, &state, from, to, folder_tree, actor)
                .map(|answer| (answer, Some(state_base_after(&state))))
        }
        (ResolveStep::Finish { from, to, folder_tree }, Some(state)) => {
            remove_join_state(&path)?;
            let base = state_base_after(&state);
            Ok((JoinAnswer::Rebased { request_ref: state.request_ref, from, to, folder_tree }, Some(base)))
        }
    }
}
