//! `mr join`: finishes a folder move a kill or a refused move left (a rebase's, a joined partner request's or an
//! undo's), saves the folder's work, joins the request to its zone on the base its folder record names, rewrites that record when the
//! join moved the request off it, and moves the folder after a rebase or a joined partner request, all under the
//! zone's join lock after recover; in an attempt folder it runs nothing.
//! No unit test: it runs git, the formatter and the op log, and takes a real lock; the unit tests in src/join
//! (request_run_for_tests, save_part_tests, join_report_tests, base_answer_part_tests, join_refusal_report_tests,
//! partners_conflicted_report_tests and move_refusal_report_tests) and in src (snapshot_settings_for_tests,
//! join_save_subject_tests) cover its decisions and reports, and finish_pending_move's header lists the cases of a
//! move left pending. The integration tests crates/cli/tests/save_join_runs/{joins,recovery,processes}.rs and
//! resolve_runs/resolve_refusals.rs cover, with the mortar binary as a child process in a throwaway repository and a
//! request folder: a request ahead of an unmoved zone fast-forwards it, exit 0 "joined", next "wait for the zone's
//! tests", and a repeat answers joined with nothing moved; unsaved edits are saved first as "<request>: work saved at
//! join" on the request branch and joined; a zone moved under the request rebases it, exit 0 "rebased", the folder's
//! files and HEAD at the new tip, next "mr join"; a conflicting zone change is exit 2 listing the files, the join
//! itself moving no ref (the save and snapshot before it may have moved the request branch and the snapshot ref, and
//! the line names them), next "mr resolve"; a local edit in the way of the folder move is exit 1 listing the paths,
//! and once moved aside the next join finishes the move; a missing zone branch is exit 1 owned by the zone's
//! coordinator; any argument is exit 4 with join's usage; an attempt folder is exit 1 naming its number with nothing
//! run; a held join lock is exit 3; a configured formatter runs over the changed files before the save.
//! save_join_runs/partner_join_tests.rs covers a request cut on a merge with a partner: unmoved, the zone takes only
//! the request's commits, the record naming the zone tip with no partners; a moved partner rebases onto a new merge the
//! record names, next "mr join", and with every partner already in the zone it is the plain rebase, the record naming
//! the zone tip with no partners; a conflicting new merge is exit 1 naming the partner and files, nothing moved.

use folders::{move_folder, rewrite_request_base, RecordedBase};
use gitio::Oid;
use oplog::KillPidProbe;
use snapshot::SaveOutcome;

use super::{
    finish_pending_move, join_refusal_report, join_report, join_save_subject, move_refusal_report, moved_before,
    partners_conflicted_report, request_run_for, save_from_formatted, snapshot_moved, JoinOutcome, RequestRun,
};
use crate::folder::folder_error_why;
use crate::lock::locked_act;
use crate::report::{failure_report, snapshot_step_failure, with_moved_first, Report};
use crate::request_base::{read_request_base, recorded_base_of, request_base_of};
use crate::shell::ShellContext;
use ::join::{BaseJoinAnswer, JoinAnswer};

const VERB: &str = "join";

/// Takes no arguments. The op key is the verb and the folder with no moves, unless --key gives one.
pub fn run_join(context: &ShellContext) -> Report {
    let run = match request_run_for(context, VERB) {
        Ok(run) => run,
        Err(report) => return report,
    };
    match locked_act(&run.git, &run.folder.common, &run.lock, &run.key, &KillPidProbe, VERB, || act(&run)) {
        Ok(report) | Err(report) => report,
    }
}

/// Finish a pending folder move, then read the record's base and partners (read_request_base), which finishing a
/// joined partner request's move may have rewritten, save, join on that base, and follow the answer. The snapshot
/// ref moved if the finished move's snapshot or the formatter's snapshot before the save moved it; the save saves
/// from the formatter's snapshot and takes none of its own. A refused join and every report after it name what the
/// save and those snapshots moved (moved_before's line). A failed snapshot of the pending move names nothing moved,
/// as nothing ran before it; a failed read of the record names what the finished move's snapshot moved.
fn act(run: &RequestRun) -> Result<Report, Report> {
    let (finished_move, finish_written) =
        finish_pending_move(run, VERB, &|error| snapshot_step_failure(VERB, &error, None))?;
    let recorded = read_request_base(run.folder).map_err(|why| {
        let (report, moved) = (failure_report(VERB, &why), moved_before(false, finish_written, &run.request_ref));
        Report { lines: with_moved_first(moved.as_deref(), report.lines), ..report }
    })?;
    let (save, snapshot_written) = save_work(run, finish_written)?;
    let moved = moved_before(save.written, snapshot_written, &run.request_ref);
    let (folder, actor, base) = (&run.folder.toplevel, &run.target.actor, request_base_of(&recorded));
    let answer = ::join::join_on_base(&run.git, folder, &run.target.zone, &run.request_ref, &base, actor)
        .map_err(|error| join_refusal_report(&error, moved.as_deref()))?;
    follow_answer(run, &answer, moved.as_deref())?;
    let request_ref = run.request_ref.clone();
    Ok(join_report(&JoinOutcome { request_ref, snapshot_written, finished_move, save, answer }))
}

/// A rebase onto the zone moves the folder. A joined partner request is rewritten to stand on the zone tip it was
/// replayed onto with no partners, and a rebase onto a new merge to that merge and its partners, before the folder
/// moves: the request ref left the merge whose files the folder's index still holds, so an unmoved folder would save
/// the partner files into the request. The record is rewritten because, after a rebase onto a new merge, a record
/// naming the old merge would make the next join replay from a base the branch no longer stands on; after a joined
/// request, status would report the old merge and partners the request no longer stands on. A conflicting new merge
/// is partners_conflicted_report. Plain's other answers move nothing. A kill or a failed write between join's op and
/// the rewrite after a rebase onto a new merge leaves the record naming the merge and partners from before the join;
/// recovering it needs join to record the base in its op.
fn follow_answer(run: &RequestRun, answer: &BaseJoinAnswer, moved: Option<&str>) -> Result<(), Report> {
    let (folder, actor, request_ref) = (&run.folder.toplevel, &run.target.actor, &run.request_ref);
    match answer {
        BaseJoinAnswer::Plain(JoinAnswer::Rebased { from, to, .. }) => {
            move_folder(&run.git, folder, from, to, true, actor, None).map_err(|error| {
                let done = format!("rebased: {request_ref} moved from {from} to {to}, replayed onto the zone");
                move_refusal_report(&error, vec![done], moved)
            })
        }
        BaseJoinAnswer::ZoneJoined { zone_ref, from, request_from, to, .. } => {
            let done = format!(
                "joined: {zone_ref} moved from {from} to {to}; {request_ref} moved from {request_from} to {to}"
            );
            let on_zone = RecordedBase { base: from.clone(), partners: Vec::new() };
            rewrite_then_move(run, &on_zone, (request_from, to), done, moved)
        }
        BaseJoinAnswer::Rebased { from, to, base, .. } => {
            let done =
                format!("rebased: {request_ref} moved from {from} to {to}, replayed onto a new merge {}", base.base);
            rewrite_then_move(run, &recorded_base_of(base), (from, to), done, moved)
        }
        BaseJoinAnswer::PartnersConflicted { zone, files } => {
            Err(partners_conflicted_report(&run.target.zone, zone, files, moved))
        }
        BaseJoinAnswer::Plain(_) => Ok(()),
    }
}

/// Rewrites the folder record to `base`, then moves the folder from `from` to `to`. `done` names what join's op
/// moved: a failed rewrite is failure_report with it, saying the folder did not move; a refused move is
/// move_refusal_report with it.
fn rewrite_then_move(
    run: &RequestRun,
    base: &RecordedBase,
    (from, to): (&Oid, &Oid),
    done: String,
    moved: Option<&str>,
) -> Result<(), Report> {
    let folder = &run.folder.toplevel;
    if let Err(error) = rewrite_request_base(&run.git, folder, base) {
        let why = folder_error_why(&error).join("; ");
        let what = format!("{done}; the folder record was not rewritten and the folder did not move: {why}");
        let report = failure_report(VERB, &what);
        return Err(Report { lines: with_moved_first(moved, report.lines), ..report });
    }
    move_folder(&run.git, folder, from, to, true, &run.target.actor, None)
        .map_err(|error| move_refusal_report(&error, vec![done], moved))
}

/// Saves the folder through save_from_formatted (the formatter's one snapshot, then the save from it), returning the
/// save and whether the snapshot ref moved by its end (`finish_written`: the finished move's snapshot moved it; or
/// the formatter's snapshot). A failure of any step names what the steps before it moved. Always run: save writes no
/// commit when the folder's tree equals HEAD's.
fn save_work(run: &RequestRun, finish_written: bool) -> Result<(SaveOutcome, bool), Report> {
    let (folder, excludes, cap) = (&run.folder.toplevel, &run.settings.excludes, run.settings.cap_bytes);
    let subject = join_save_subject(&run.target.request);
    let (saved, _, written) =
        save_from_formatted(&run.git, folder, &run.target, &subject, excludes, cap, run.settings.formatter.as_ref())
            .map_err(|(error, written)| {
                let moved = moved_before(false, finish_written || written, &run.request_ref);
                snapshot_step_failure(VERB, &error, moved.as_deref())
            })?;
    let moved = snapshot_moved(finish_written || written, &saved);
    Ok((saved.outcome, moved))
}
