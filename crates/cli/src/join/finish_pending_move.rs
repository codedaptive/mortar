//! Finishes the folder move of a request's last join rebase (S-86) or of an undo that put a rebased branch back
//! (S-143), if one is pending: a kill or a refused move left it, or it is done and repeats as a no-op.
//! Shell API: run_join, run_resolve, run_save and run_undo call it.
//! It passes the op's recorded folder_tree as move_folder's index_tree, None for a plain join's rebase and for an
//! undo's move (S-87(3), S-89).
//! No unit test: it reads the op log, snapshots and moves the folder with git (S-5); find_pending_move_tests covers
//! the decision and crates/cli/tests/move_refusal_report_tests.rs its refusal. Nagatha's integration group is to
//! cover, with the mortar binary as a child process in a throwaway repository and a request folder: a kill after
//! join's rebase op and before the folder move (the branch ahead of the folder) is finished by the next join before
//! it saves, with the finished move in its report and the folder's work in the request's snapshot ref; a local edit
//! in the way of the pending move is exit 1 listing the paths, and once moved aside the next join finishes it; with
//! no move pending it moves nothing; a resolve whose folder move is refused (an edit on a path the zone changed), the
//! files put back to the folder's version, then mortar join finishes the move from the op's folder_tree; a kill
//! after resolve's final op and before the folder move is finished by the next join; an undo of a join rebase whose
//! folder move was refused is finished by the next save, undo or join, the folder back at the tip before the rebase.

use folders::move_folder;
use snapshot::snapshot;

use super::{move_refusal_report, read_pending_move, snapshot_failure, PendingMove, RequestRun};
use crate::report::Report;

/// The pending move, finished: snapshot the folder's work first (S-74), then move the folder from the move's old tip
/// to its new one, saved true from that snapshot, with the op's folder_tree as the index tree (None for a plain
/// join's rebase and for an undo's move). None when no move is pending. A failed read or snapshot is failure_report
/// and a refused move move_refusal_report, each for `verb`.
pub fn finish_pending_move(run: &RequestRun, verb: &str) -> Result<Option<PendingMove>, Report> {
    let Some(pending) = read_pending_move(&run.git, &run.folder.common, &run.request_ref, verb)? else {
        return Ok(None);
    };
    let (excludes, cap) = (&run.settings.excludes, run.settings.cap_bytes);
    snapshot(&run.git, &run.folder.toplevel, &run.target, excludes, cap)
        .map_err(|error| snapshot_failure(verb, error))?;
    move_folder(
        &run.git,
        &run.folder.toplevel,
        &pending.old,
        &pending.new,
        true,
        &run.target.actor,
        pending.folder_tree.as_ref(),
    )
    .map_err(|error| Report { verb: verb.to_string(), ..move_refusal_report(&error, Vec::new()) })?;
    Ok(Some(pending))
}
