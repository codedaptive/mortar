//! `mr resolve`: finishes a folder move a kill or a refused move left (a rebase's, a joined partner request's or an
//! undo's), snapshots the folder, resolves the join paused in it, rewrites the folder record to the base the paused
//! join kept and moves the folder after the rebase that completes it, all under the zone's join lock after recover; in
//! an attempt folder it runs nothing.
//! No unit test: it runs git and the op log, and takes a real lock; the unit tests in src
//! (request_run_for_tests, resolve_report_tests, resolve_refusal_report_tests, resolve_move_refusal_report_tests and
//! run_resolve_tests, which covers an attempt folder and an argument, both returning before any lock or git run) cover
//! its decisions and reports, and finish_pending_move's header lists the cases of a move left pending. The integration
//! tests crates/cli/tests/resolve_runs/{resolve_first_runs,resolve_finishes,resolve_unusual,resolve_refusals,
//! resolve_catch_up,resolve_base_tests}.rs cover, with the mortar binary as a child process in a throwaway repository
//! and a request folder: no paused join is exit 0 nothing-to-resolve; the first run writes the files and lists
//! skipped and gitlinks, exit 2; a next run with markers is exit 2 naming files and lines; a next run resolved and clean is Rebased, exit 0,
//! the folder's files and HEAD at the new tip with the resolved content and edits outside the resolved paths kept;
//! conflicted again on a later commit is exit 2 listing the files; an edit on a path the zone changed is exit 1 naming
//! the paths and the snapshot ref; a kill after join's final op and before the folder move is finished by the next
//! resolve or join (from the op's folder_tree); a plain resolve records the zone tip as the record's base, and a kill
//! after resolve's final op and before the record rewrite is finished by the next save, the record naming the zone tip;
//! a held lock is exit 3; an attempt folder is exit 1 with nothing run;
//! any argument is exit 4. crates/cli/tests/save_join_runs/partner_join_tests.rs covers a request cut on a merge with a
//! partner whose join paused on the unchanged zone: the resolve that completes it is Rebased, says the request left
//! its set's merge with left_partners naming the partner and next "mr join", and the record names the zone tip with
//! no partners.

use folders::plan::snap_ref;
use folders::move_folder;
use oplog::KillPidProbe;
use snapshot::snapshot;

use super::{left_set_report, resolve_move_refusal_report, resolve_refusal_report, resolve_report, ResolveOutcome};
use crate::join::{finish_pending_move, moved_before, request_run_for, rewrite_record_to, RequestRun};
use crate::lock::locked_act;
use crate::report::{failure_report, snapshot_step_failure, with_moved_first, Report};
use crate::request_base::{read_request_base, recorded_base_of};
use crate::shell::ShellContext;

const VERB: &str = "resolve";

/// Takes no arguments. The op key is the verb and the folder with no moves, unless --key gives one. A
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

/// Finish a pending folder move, then read the record's base and partners (read_request_base), which finishing the
/// move may have rewritten, snapshot the folder (no formatter: the files may hold markers), resolve with the
/// snapshot's tree, and after a rebase rewrite the record to the base the paused join kept (when it kept one) and move
/// the folder: saved from that snapshot, from the answer's folder_tree. A rebase onto the zone tip with no partners,
/// for a request whose record named partners, is left_set_report: the request stands without them now (S-233). A
/// failed read of the record is failure_report naming what the finished move's snapshot moved (moved_before's line);
/// the rewrite is rewrite_record_to, no write when the record already holds that base, and its failure is
/// failure_report naming the rebase and moved_before's line, the folder not moved. A kill or a
/// failed write between resolve's final op and the rewrite leaves the record naming the base and partners from before
/// the resolve, with the folder move pending; resolve's final op records the base before and after
/// (::join::recorded_base), and the next join's or resolve's finish_pending_move rewrites the record to the base
/// after before it moves the folder.
/// The snapshot runs every time, so the first run keeps the folder's files before resolve writes over them. A
/// refusal from resolve and a refused folder move after the rebase name the snapshot ref when either snapshot (the
/// pending move's or this one) wrote a commit (moved_before's line). A failure of this snapshot names the snapshot
/// ref when the pending move's snapshot wrote; a failure of the pending move's snapshot names nothing moved, as
/// nothing ran before it.
/// When the finished move changes no folder file, this snapshot sees the same tree as finish's and writes nothing, so
/// finish's flag is then the only source of that fact.
fn act(run: &RequestRun, snapshot_ref: &str) -> Result<Report, Report> {
    let (finished_move, finish_written) =
        finish_pending_move(run, VERB, &|error| snapshot_step_failure(VERB, &error, None))?;
    let recorded = read_request_base(run.folder).map_err(|why| {
        let (report, moved) = (failure_report(VERB, &why), moved_before(false, finish_written, &run.request_ref));
        Report { lines: with_moved_first(moved.as_deref(), report.lines), ..report }
    })?;
    let (folder, actor) = (&run.folder.toplevel, &run.target.actor);
    let taken = snapshot(&run.git, folder, &run.target, &run.settings.excludes, run.settings.cap_bytes)
        .map_err(|error| {
            snapshot_step_failure(VERB, &error, moved_before(false, finish_written, &run.request_ref).as_deref())
        })?;
    let snapshot_written = finish_written || taken.written;
    let moved = moved_before(false, snapshot_written, &run.request_ref);
    let (answer, base) = ::join::resolve_on_base(&run.git, folder, &taken.tree, actor)
        .map_err(|error| resolve_refusal_report(&error, moved.as_deref()))?;
    if let ::join::JoinAnswer::Rebased { from, to, folder_tree, .. } = &answer {
        let done = format!("rebased: {} moved from {from} to {to}", run.request_ref);
        rewrite_record_to(run, VERB, Ok(base.as_ref().map(recorded_base_of)), Some(&done), moved.as_deref())?;
        move_folder(&run.git, folder, from, to, true, actor, folder_tree.as_ref()).map_err(|error| {
            resolve_move_refusal_report(&error, snapshot_ref, &taken.commit, vec![done], moved.as_deref())
        })?;
    }
    let report = resolve_report(&ResolveOutcome {
        request_ref: run.request_ref.clone(),
        finished_move,
        snapshot_ref: snapshot_ref.to_string(),
        snapshot_written,
        snapshot_commit: taken.commit,
        answer,
    });
    let left: Vec<String> = match &base {
        Some(base) if base.partners.is_empty() => recorded.partners.into_iter().map(|partner| partner.zone).collect(),
        _ => Vec::new(),
    };
    Ok(if left.is_empty() { report } else { left_set_report(report, &left) })
}
