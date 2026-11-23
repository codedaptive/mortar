//! Finishes the folder move of a request's last join rebase, of its joined partner request, or of an undo of a plain
//! join rebase that put the rebased branch back, if one is pending: a kill or a refused move left it, or it is done
//! and repeats as a no-op. An undo of a resolve step is no pending move: undo set the index itself and kept the files.
//! For a joined partner request it also finishes the folder record's rewrite to the zone tip with no partners.
//! Shell API: run_join, run_resolve, run_save, run_undo and undo's folder_step call it.
//! It passes the op's recorded folder_tree as move_folder's index_tree, None for a plain join's rebase, for a
//! joined partner request and for an undo's move.
//! No unit test: it reads the op log and the folder record, snapshots and moves the folder with git;
//! find_pending_move_tests and joined_zone_base_tests cover the decisions and move_refusal_report_tests its refusal. The integration tests
//! crates/cli/tests/save_join_runs/{joins,recovery,processes,partner_join_tests}.rs,
//! resolve_runs/{resolve_refusals,resolve_catch_up}.rs and across_kits/undo_save_join_resolve/{rebases,killed,resolve_undo}.rs cover, with the mortar binary as a child
//! process in a throwaway repository and a request folder: a kill after
//! join's rebase op and before the folder move (the branch ahead of the folder) is finished by the next join before
//! it saves, with the finished move in its report and the folder's work in the request's snapshot ref; a local edit
//! in the way of the pending move is exit 1 listing the paths, and once moved aside the next join finishes it; with
//! no move pending it moves nothing; a resolve whose folder move is refused (an edit on a path the zone changed), the
//! files put back to the folder's version, then mr join finishes the move from the op's folder_tree; a kill
//! after resolve's final op, or after a partner request's join-zone-request op
//! (partner_join_killed_before_folder_move_finishes_on_repeat), and before the folder move is finished by the next
//! join; a kill after a partner request's join-zone-request op and before the record rewrite is finished by the next
//! join, the record naming the zone tip with no partners (partner_join_killed_before_record_rewrite_finishes_on_repeat);
//! an undo of a plain join rebase
//! whose folder move was refused is finished by the next save, undo or join, the folder back at the tip before the
//! rebase; after an undo of a resolve step the next save, join or resolve moves no folder files.

use folders::{move_folder, rewrite_request_base, RecordedBase};
use gitio::Oid;
use snapshot::{snapshot, SnapshotError};

use super::{move_refusal_report, moved_before, read_pending_move, PendingMove, RequestRun};
use crate::folder::folder_error_why;
use crate::report::{failure_report, Report};
use crate::request_base::read_request_base;

/// Undo's verb: its reports name no ref the snapshot moved, so its refused move carries no moved line.
const UNDO: &str = "undo";

/// The pending move, finished: snapshot the folder's work first, then move the folder from the move's old tip
/// to its new one, saved true from that snapshot, with the op's folder_tree as the index tree (None for a plain
/// join's rebase, a joined partner request and an undo's move). Gives the move and whether its snapshot moved the
/// snapshot ref; (None, false) when no move is pending. A failed read is failure_report and a refused move move_refusal_report, each for
/// `verb`; a refused move whose snapshot wrote starts with moved_before's line for the snapshot ref, except under undo.
/// A failed snapshot is what `on_snapshot_error` makes of it.
/// Before the snapshot, a joined partner request's move (read_pending_move's joined zone base) whose folder record
/// still names partners has its record rewritten to that zone tip with no partners (finish_record_rewrite).
pub fn finish_pending_move(
    run: &RequestRun,
    verb: &str,
    on_snapshot_error: &dyn Fn(SnapshotError) -> Report,
) -> Result<(Option<PendingMove>, bool), Report> {
    let Some((pending, joined_base)) = read_pending_move(&run.git, &run.folder.common, &run.request_ref, verb)? else {
        return Ok((None, false));
    };
    if let Some(base) = joined_base {
        finish_record_rewrite(run, verb, &base)?;
    }
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

/// Rewrites the folder record to `base` with no partners when it still names partners; a record with none (the
/// normal path already rewrote it) is left unwritten. A failed read or rewrite is failure_report for `verb` naming
/// the folder, with nothing moved.
fn finish_record_rewrite(run: &RequestRun, verb: &str, base: &Oid) -> Result<(), Report> {
    let folder = &run.folder.toplevel;
    let failed = |why: String| {
        let what = format!(
            "the folder record of {} was not rewritten to base {base} and the folder did not move: {why}",
            folder.display()
        );
        failure_report(verb, &what)
    };
    if read_request_base(run.folder).map_err(&failed)?.partners.is_empty() {
        return Ok(());
    }
    let on_zone = RecordedBase { base: base.clone(), partners: Vec::new() };
    rewrite_request_base(&run.git, folder, &on_zone).map_err(|error| failed(folder_error_why(&error).join("; ")))?;
    Ok(())
}
