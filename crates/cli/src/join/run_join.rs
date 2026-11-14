//! `mortar join`: finishes a folder move a kill left, saves the folder's work, joins the request to its zone and
//! moves the folder after a rebase, all under the zone's join lock after recover (S-72, S-86, S-47, S-49); in an
//! attempt folder it runs nothing (S-91).
//! No unit test: it runs git, the formatter and the op log, and takes a real lock (S-5); the tests in crates/cli/tests
//! (request_run_for_tests, save_part_tests, join_report_tests, join_refusal_report_tests and move_refusal_report_tests)
//! and in src (snapshot_settings_for_tests, join_save_subject_tests) cover its decisions and reports, and
//! finish_pending_move's header lists the cases of a move a kill left. Nagatha's integration group is to cover, with
//! the mortar binary as a child process in a throwaway repository and a request folder: a request ahead of an unmoved
//! zone fast-forwards it, exit 0 "joined", next "wait for the zone's tests", and a repeat answers joined with nothing
//! moved; unsaved edits are saved first as "<request>: work saved at join" on the request branch and joined; a zone
//! moved under the request rebases it, exit 0 "rebased", the folder's files and HEAD at the new tip, next "mortar
//! join"; a conflicting zone change is exit 2 listing the files, no ref moved, next "mortar resolve"; a local edit in
//! the way of the folder move is exit 1 listing the paths, and once moved aside the next join finishes the move; a
//! missing zone branch is exit 1 owned by the zone's coordinator; any argument is exit 4 with join's usage; an attempt
//! folder is exit 1 naming its number with nothing run; a held join lock is exit 3; a configured formatter runs over
//! the changed files before the save.

use folders::move_folder;
use oplog::KillPidProbe;
use snapshot::{save, snapshot_formatted, SaveOutcome};

use super::{
    finish_pending_move, join_refusal_report, join_report, join_save_subject, move_refusal_report, request_run_for,
    snapshot_failure, JoinOutcome, RequestRun,
};
use crate::lock::locked_act;
use crate::report::Report;
use crate::shell::ShellContext;

const VERB: &str = "join";

/// Takes no arguments. The op key is the verb and the folder with no moves (S-84(4)), unless --key gives one.
pub fn run_join(context: &ShellContext) -> Report {
    let run = match request_run_for(context, VERB) {
        Ok(run) => run,
        Err(report) => return report,
    };
    match locked_act(&run.git, &run.folder.common, &run.lock, &run.key, &KillPidProbe, VERB, || act(&run)) {
        Ok(report) | Err(report) => report,
    }
}

/// Finish a pending folder move, save, join, and move the folder after a rebase (saved: the save just ran).
fn act(run: &RequestRun) -> Result<Report, Report> {
    let finished_move = finish_pending_move(run, VERB)?;
    let save = save_work(run)?;
    let (folder, actor) = (&run.folder.toplevel, &run.target.actor);
    let answer = ::join::join(&run.git, folder, &run.target.zone, &run.request_ref, actor)
        .map_err(|error| join_refusal_report(&error))?;
    if let ::join::JoinAnswer::Rebased { from, to, .. } = &answer {
        move_folder(&run.git, folder, from, to, true, actor, None).map_err(|error| {
            let done = format!("rebased: {} moved from {from} to {to}, replayed onto the zone", run.request_ref);
            move_refusal_report(&error, vec![done])
        })?;
    }
    Ok(join_report(&JoinOutcome { request_ref: run.request_ref.clone(), finished_move, save, answer }))
}

/// Runs the formatter with a snapshot, then saves (S-72). Always run: save writes nothing when the folder's tree
/// equals HEAD's.
fn save_work(run: &RequestRun) -> Result<SaveOutcome, Report> {
    let (folder, excludes, cap) = (&run.folder.toplevel, &run.settings.excludes, run.settings.cap_bytes);
    snapshot_formatted(&run.git, folder, &run.target, excludes, cap, run.settings.formatter.as_ref())
        .map_err(|error| snapshot_failure(VERB, error))?;
    save(&run.git, folder, &run.target, &join_save_subject(&run.target.request), excludes, cap)
        .map_err(|error| snapshot_failure(VERB, error))
}
