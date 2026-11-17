//! `mr resolve`: finishes a folder move a kill or a refused move left (a rebase's or an undo's, S-143),
//! snapshots the folder, resolves the join paused in it and moves the folder after the rebase that completes it, all
//! under the zone's join lock after recover (S-50, S-64, S-86, S-87, S-88); in an attempt folder it runs nothing
//! (S-91).
//! No unit test: it runs git and the op log, and takes a real lock (S-5); the tests in crates/cli/tests
//! (request_run_for_tests, resolve_report_tests, resolve_refusal_report_tests, resolve_move_refusal_report_tests and
//! run_resolve_tests, which covers an attempt folder and an argument, both returning before any lock or git run) cover
//! its decisions and reports, and finish_pending_move's header lists the cases of a move left pending. Nagatha's
//! integration group is to cover, with the mortar binary as a child process in a throwaway repository and a request
//! folder: no paused join is exit 0 nothing-to-resolve; the first run writes the files and lists skipped and gitlinks,
//! exit 2; a next run with markers is exit 2 naming files and lines; a next run resolved and clean is Rebased, exit 0,
//! the folder's files and HEAD at the new tip with the resolved content and edits outside the resolved paths kept;
//! conflicted again on a later commit is exit 2 listing the files; an edit on a path the zone changed is exit 1 naming
//! the paths and the snapshot ref; a kill after join's final op and before the folder move is finished by the next
//! resolve or join (from the op's folder_tree); a held lock is exit 3; an attempt folder is exit 1 with nothing run;
//! any argument is exit 4.

use folders::move_folder;
use folders::plan::snap_ref;
use oplog::KillPidProbe;
use snapshot::snapshot;

use super::{resolve_move_refusal_report, resolve_refusal_report, resolve_report, ResolveOutcome};
use crate::join::{finish_pending_move, request_run_for, snapshot_failure, RequestRun};
use crate::lock::locked_act;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "resolve";

/// Takes no arguments. The op key is the verb and the folder with no moves (S-84(4)), unless --key gives one. A
/// snapshot ref that cannot be named is failure_report, before the lock.
pub fn run_resolve(context: &ShellContext) -> Report {
    let run = match request_run_for(context, VERB) {
        Ok(run) => run,
        Err(report) => return report,
    };
    let snapshot_ref = match snap_ref(&run.target.zone, &run.target.request) {
        Ok(name) => name,
        Err(error) => return failure_report(VERB, &error.to_string()),
    };
    match locked_act(&run.git, &run.folder.common, &run.lock, &run.key, &KillPidProbe, VERB, || {
        act(&run, &snapshot_ref)
    }) {
        Ok(report) | Err(report) => report,
    }
}

/// Finish a pending folder move, snapshot the folder (no formatter: the files may hold markers), resolve with the
/// snapshot's tree, and move the folder after a rebase: saved from that snapshot, from the answer's folder_tree.
/// The snapshot runs every time, so the first run keeps the folder's files before resolve writes over them.
fn act(run: &RequestRun, snapshot_ref: &str) -> Result<Report, Report> {
    let finished_move = finish_pending_move(run, VERB)?;
    let (folder, actor) = (&run.folder.toplevel, &run.target.actor);
    let taken = snapshot(&run.git, folder, &run.target, &run.settings.excludes, run.settings.cap_bytes)
        .map_err(|error| snapshot_failure(VERB, error))?;
    let answer =
        ::join::resolve(&run.git, folder, &taken.tree, actor).map_err(|error| resolve_refusal_report(&error))?;
    if let ::join::JoinAnswer::Rebased { from, to, folder_tree, .. } = &answer {
        move_folder(&run.git, folder, from, to, true, actor, folder_tree.as_ref()).map_err(|error| {
            let done = format!("rebased: {} moved from {from} to {to}", run.request_ref);
            resolve_move_refusal_report(&error, snapshot_ref, &taken.commit, vec![done])
        })?;
    }
    Ok(resolve_report(&ResolveOutcome {
        request_ref: run.request_ref.clone(),
        finished_move,
        snapshot_ref: snapshot_ref.to_string(),
        snapshot_commit: taken.commit,
        answer,
    }))
}
