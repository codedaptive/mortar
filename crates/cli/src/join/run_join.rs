//! `mr join`: finishes a folder move a kill or a refused move left (a rebase's or an undo's), saves the
//! folder's work, joins the request to its zone and moves the folder after a rebase, all under the zone's join lock
//! after recover; in an attempt folder it runs nothing.
//! No unit test: it runs git, the formatter and the op log, and takes a real lock; the unit tests in src/join
//! (request_run_for_tests, save_part_tests, join_report_tests, join_refusal_report_tests and move_refusal_report_tests)
//! and in src (snapshot_settings_for_tests, join_save_subject_tests) cover its decisions and reports, and
//! finish_pending_move's header lists the cases of a move left pending. The integration tests
//! crates/cli/tests/save_join_runs/{joins,recovery,processes}.rs and resolve_runs/resolve_refusals.rs cover, with the
//! mortar binary as a child process in a throwaway repository and a request folder: a request ahead of an unmoved
//! zone fast-forwards it, exit 0 "joined", next "wait for the zone's tests", and a repeat answers joined with nothing
//! moved; unsaved edits are saved first as "<request>: work saved at join" on the request branch and joined; a zone
//! moved under the request rebases it, exit 0 "rebased", the folder's files and HEAD at the new tip, next "mr join";
//! a conflicting zone change is exit 2 listing the files, the join itself moving no ref (the save and snapshot before
//! it may have moved the request branch and the snapshot ref, and the line names them), next "mr resolve"; a local
//! edit in the way of the folder move is exit 1 listing the paths, and once moved aside the next join finishes the
//! move; a missing zone branch is exit 1 owned by the zone's coordinator; any argument is exit 4 with join's usage;
//! an attempt folder is exit 1 naming its number with nothing run; a held join lock is exit 3; a configured formatter
//! runs over the changed files before the save.

use folders::move_folder;
use oplog::KillPidProbe;
use snapshot::{save_with_snapshot, snapshot_formatted, SaveOutcome, SnapshotError};

use super::{
    finish_pending_move, join_refusal_report, join_report, join_save_subject, move_refusal_report, moved_before,
    request_run_for, snapshot_moved, JoinOutcome, RequestRun,
};
use crate::lock::locked_act;
use crate::report::Report;
use crate::report::snapshot_step_failure;
use crate::shell::ShellContext;

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

/// Finish a pending folder move, save, join, and move the folder after a rebase (saved: the save just ran). The
/// snapshot ref moved if the finished move's snapshot, the formatter's snapshot before the save or save's own
/// snapshot step moved it; a refused join and a refused folder move after the rebase name what the save and those
/// snapshots moved (moved_before's line). A failed snapshot of the pending move is reported with no moved line, as
/// nothing ran before it.
fn act(run: &RequestRun) -> Result<Report, Report> {
    let (finished_move, finish_written) =
        finish_pending_move(run, VERB, &|error| snapshot_step_failure(VERB, &error, None))?;
    let (save, snapshot_written) = save_work(run, finish_written)?;
    let (folder, actor) = (&run.folder.toplevel, &run.target.actor);
    let moved = moved_before(save.written, snapshot_written, &run.request_ref);
    let answer = ::join::join(&run.git, folder, &run.target.zone, &run.request_ref, actor)
        .map_err(|error| join_refusal_report(&error, moved.as_deref()))?;
    if let ::join::JoinAnswer::Rebased { from, to, .. } = &answer {
        move_folder(&run.git, folder, from, to, true, actor, None).map_err(|error| {
            let done = format!("rebased: {} moved from {from} to {to}, replayed onto the zone", run.request_ref);
            move_refusal_report(&error, vec![done], moved.as_deref())
        })?;
    }
    Ok(join_report(&JoinOutcome {
        request_ref: run.request_ref.clone(),
        snapshot_written,
        finished_move,
        save,
        answer,
    }))
}

/// Runs the formatter with a snapshot, then saves, returning the save and whether the snapshot ref moved by its end
/// (`finish_written`: the finished move's snapshot moved it; or the formatter's snapshot, or save's own snapshot
/// step). A failure of either step names what the steps before it moved. Always run: save writes no commit when the
/// folder's tree equals HEAD's.
fn save_work(run: &RequestRun, finish_written: bool) -> Result<(SaveOutcome, bool), Report> {
    let (folder, excludes, cap) = (&run.folder.toplevel, &run.settings.excludes, run.settings.cap_bytes);
    let failure = |error: SnapshotError, earlier: bool| {
        snapshot_step_failure(VERB, &error, moved_before(false, earlier, &run.request_ref).as_deref())
    };
    let taken = snapshot_formatted(&run.git, folder, &run.target, excludes, cap, run.settings.formatter.as_ref())
        .map_err(|error| failure(error, finish_written))?;
    let earlier = finish_written || taken.written;
    let subject = join_save_subject(&run.target.request);
    let saved = save_with_snapshot(&run.git, folder, &run.target, &subject, excludes, cap)
        .map_err(|error| failure(error, earlier))?;
    let moved = snapshot_moved(earlier, &saved);
    Ok((saved.outcome, moved))
}
