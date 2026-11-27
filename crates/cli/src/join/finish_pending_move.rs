//! Finishes the folder move of a request's last join rebase, of its joined partner request, or of an undo of a join
//! rebase that is not a resolve step (the undo put the rebased branch back), if one is pending: a kill or a refused
//! move left it, or it is done and repeats as a no-op. An undo of a resolve step is no pending move: undo set the
//! index itself and kept the files.
//! Before the move it finishes the folder record's rewrite to the base the join op recorded (pending_base): for a
//! plain rebase (join's or resolve's) the zone tip replayed onto with no partners, for a partner rebase the new merge
//! and its partners, for a joined partner request the zone's tip before the join, with no partners; for an undo of a
//! join rebase, a partner one included, the base before that rebase. An op that recorded no base leaves the record
//! as it is, except a joined partner request's, which joined_zone_base reads from its moves.
//! Shell API: run_join, run_resolve, run_save, run_undo and undo's folder_step call it.
//! It passes the op's recorded folder_tree as move_folder's index_tree, None for a plain join's rebase, for a
//! joined partner request and for an undo's move.
//! No unit test: it reads the op log and the folder record, snapshots and moves the folder with git;
//! find_pending_move_tests and pending_base_tests cover the decisions and move_refusal_report_tests its refusal. The
//! integration tests crates/cli/tests/save_join_runs/{joins,recovery,processes,partner_join_tests,plain_base_tests}.rs,
//! resolve_runs/{resolve_refusals,resolve_catch_up,resolve_base_tests}.rs and
//! across_kits/undo_save_join_resolve/{rebases,killed,resolve_undo,base_undo}.rs cover, with the mortar binary as a
//! child process in a throwaway repository and a request folder: a kill after
//! join's rebase op and before the folder move (the branch ahead of the folder) is finished by the next join before
//! it saves, with the finished move in its report and the folder's work in the request's snapshot ref; a local edit
//! in the way of the pending move is exit 3 listing the paths, and once moved aside the next join finishes it; with
//! no move pending it moves nothing; a resolve whose folder move is refused (an edit on a path the zone changed), the
//! files put back to the folder's version, then mr join finishes the move from the op's folder_tree; a kill
//! after resolve's final op, or after a partner request's join-zone-request op
//! (partner_join_killed_before_folder_move_finishes_on_repeat), and before the folder move is finished by the next
//! join; a kill after a partner request's join-zone-request op and before the record rewrite is finished by the next
//! join, the record naming the zone's tip before the join, with no partners
//! (partner_join_killed_before_record_rewrite_finishes_on_repeat); a kill after a plain join's rebase op and before
//! the folder move is finished by the next join, the record naming the zone tip
//! (plain_base_recovers_killed_rebase); a kill after a partner rebase's op and before the folder moves is finished
//! by the next save, the record naming the new merge (partner_base_recovers_killed_rebase); a kill after resolve's
//! final op and before the record rewrite is finished by the next save, the record naming the zone tip
//! (resolve_base_recovers_kill_before_record_rewrite); an undo of a plain join rebase puts the record's base back
//! (base_undo_plain_join_and_rejoin), and an undo of a partner rebase the cut merge and its partners
//! (partner_join_undo_restores_cut_boundary_and_rejoin); an undo whose folder move was refused is
//! finished by the next save, undo or join, the folder back at the tip before the rebase; after an undo of a resolve
//! step the next save, join or resolve moves no folder files.

use folders::move_folder;
use snapshot::{snapshot, SnapshotError};

use super::{move_refusal_report, moved_before, read_pending_move, rewrite_record_to, PendingMove, RequestRun};
use crate::report::{failure_report, Report};

/// Undo's verb: its reports name no ref the snapshot moved, so its refused move carries no moved line.
const UNDO: &str = "undo";

/// The pending move, finished: rewrite the folder record to the move's base, snapshot the folder's work, then move
/// the folder from the move's old tip to its new one, saved true from that snapshot, with the op's folder_tree as the
/// index tree (None for a plain join's rebase, a joined partner request and an undo's move). Gives the move and
/// whether its snapshot moved the snapshot ref; (None, false) when no move is pending.
/// The rewrite is rewrite_record_to with read_pending_move's base, none when the op recorded none; a recorded base
/// that cannot be read, or a failed rewrite, is its failure_report for `verb` naming the folder, with no moved line.
/// A failed read of the op log or the request ref is failure_report and a refused move move_refusal_report, each for
/// `verb`; a refused move whose
/// snapshot wrote starts with moved_before's line for the snapshot ref, except under undo.
/// A failed snapshot is what `on_snapshot_error` makes of it.
pub fn finish_pending_move(
    run: &RequestRun,
    verb: &str,
    on_snapshot_error: &dyn Fn(SnapshotError) -> Report,
) -> Result<(Option<PendingMove>, bool), Report> {
    let found =
        read_pending_move(&run.git, &run.folder.common, &run.request_ref).map_err(|why| failure_report(verb, &why))?;
    let Some((pending, base)) = found else {
        return Ok((None, false));
    };
    rewrite_record_to(run, verb, base, None, None)?;
    let (excludes, cap) = (&run.settings.excludes, run.settings.cap_bytes);
    let taken = snapshot(&run.git, &run.folder.toplevel, &run.target, excludes, cap).map_err(on_snapshot_error)?;
    move_folder(
        &run.git,
        &run.folder.toplevel,
        &pending.old,
        &pending.new,
        true,
        &run.target.actor,
        pending.folder_tree.as_ref(),
    )
    .map_err(|error| {
        let moved = moved_before(false, taken.written && verb != UNDO, &run.request_ref);
        Report { verb: verb.to_string(), ..move_refusal_report(&error, Vec::new(), moved.as_deref()) }
    })?;
    Ok((Some(pending), taken.written))
}
