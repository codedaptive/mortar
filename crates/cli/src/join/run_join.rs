//! `mr join`: finishes a folder move a kill or a refused move left (a rebase's, a joined partner request's or an
//! undo's), saves the folder's work, joins the request to its zone on the base its folder record names, rewrites that
//! record when the join replayed the request onto a zone tip, and moves the folder after a rebase or a joined partner
//! request, all under the zone's join lock after recover; in an attempt folder it runs nothing.
//! No unit test: it runs git, the formatter and the op log, and takes a real lock; the unit tests in src/join
//! (request_run_for_tests, save_part_tests, join_report_tests, base_answer_part_tests, join_refusal_report_tests,
//! partners_conflicted_report_tests and move_refusal_report_tests) and in src (snapshot_settings_for_tests,
//! join_save_subject_tests) cover its decisions and reports, and finish_pending_move's header lists the cases of a
//! move left pending. The integration tests crates/cli/tests/save_join_runs/{joins,recovery,processes}.rs and
//! resolve_runs/resolve_refusals.rs cover, with the mortar binary as a child process in a throwaway repository and a
//! request folder: a request ahead of an unmoved zone fast-forwards it, exit 0 "joined", next "wait for the zone's
//! tests", and a repeat answers joined with nothing moved; unsaved edits are saved first as "<request>: work saved at
//! join" on the request branch and joined; a zone moved under the request rebases it, exit 0 "rebased", the folder's
//! files and HEAD at the new tip, next "mr join"; a conflicting zone change is exit 1 listing the files, the join
//! itself moving no ref (the save and snapshot before it may have moved the request branch and the snapshot ref, and
//! the line names them), next "mr resolve"; a pre-commit or commit-msg hook refusing the save is exit 3 refused with
//! the hook's output, nothing saved or joined and neither the request nor the zone moved
//! (joins.rs::join_hook_refusals_leave_request_and_zone_unmoved); a local edit in the way of the folder move is exit 3
//! listing the paths, and once moved aside the next join finishes the move; a missing zone branch is exit 3 owned by the zone's
//! coordinator; any argument is exit 2 with join's usage; an attempt folder is exit 3 naming its number with nothing
//! run; a held join lock is exit 4; a configured formatter runs over the changed files before the save.

use oplog::KillPidProbe;
use snapshot::{SaveHooksError, SaveOutcome};

use super::follow_answer::follow_answer;
use super::{
    finish_pending_move, join_refusal_report, join_report, join_save_subject, moved_before, request_run_for,
    save_from_formatted, snapshot_moved, JoinOutcome, RequestRun,
};
use crate::lock::locked_act;
use crate::report::{failure_report, hook_refusal_report, snapshot_step_failure, with_moved_first, Report};
use crate::request_base::{read_request_base, request_base_of};
use crate::shell::ShellContext;

const VERB: &str = "join";

/// What a hook's refusal of join's save left: the refused commit is the save's, before any join step, while the
/// snapshot steps before it may have written.
const JOIN_NOTHING_MOVED: &str =
    "no save commit was written, the branch did not move and nothing was joined; the snapshot ref may have moved";

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

/// Finish a pending folder move, then read the record's base and partners (read_request_base), which finishing that
/// move may have rewritten to the base its join op recorded, save, join on that base, and follow the answer. The
/// snapshot ref moved if the finished move's snapshot or the formatter's snapshot before the save moved it; the save
/// saves from the formatter's snapshot and takes none of its own, and runs the folder's commit hooks. A refused join,
/// a hook's refusal of the save and every report after the save name what the save and those snapshots moved
/// (moved_before's line). A failed snapshot of the pending move names nothing moved,
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

/// Saves the folder through save_from_formatted (the formatter's one snapshot, then the save from it), returning the
/// save and whether the snapshot ref moved by its end (`finish_written`: the finished move's snapshot moved it; or
/// the formatter's snapshot). A failure of any step names what the steps before it moved: a snapshot or save error
/// through snapshot_step_failure, a hook's refusal through hook_refusal_report with that line first. Always run: save
/// writes no commit when the folder's tree equals HEAD's.
fn save_work(run: &RequestRun, finish_written: bool) -> Result<(SaveOutcome, bool), Report> {
    let (folder, excludes, cap) = (&run.folder.toplevel, &run.settings.excludes, run.settings.cap_bytes);
    let subject = join_save_subject(&run.target.request);
    let (saved, _, written) =
        save_from_formatted(&run.git, folder, &run.target, &subject, excludes, cap, run.settings.formatter.as_ref())
            .map_err(|(error, written)| {
                let moved = moved_before(false, finish_written || written, &run.request_ref);
                match error {
                    SaveHooksError::Snapshot(error) => snapshot_step_failure(VERB, &error, moved.as_deref()),
                    SaveHooksError::HookRefused(refusal) => {
                        let report = hook_refusal_report(VERB, &refusal, JOIN_NOTHING_MOVED);
                        Report { lines: with_moved_first(moved.as_deref(), report.lines), ..report }
                    }
                }
            })?;
    let moved = snapshot_moved(finish_written || written, &saved);
    Ok((saved.outcome, moved))
}
