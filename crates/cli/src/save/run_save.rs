//! run_save: `mr save "<sentence>"`, the agent verb that commits its folder's files on the folder's branch
//! (the request's branch, or attempt k's try branch).
//! No unit test: it takes the lock, runs git, the formatter and the op log; save_sentence, save_run_for,
//! snapshot_target_for, save_report, save_step_failure, find_pending_move, snapshot_settings_for and op_key are its
//! decisions and have their tests.
//! The integration tests crates/cli/tests/save_join_runs/{saves,processes,recovery,joins}.rs and
//! across_kits/undo_save_join_resolve/{rebases,killed}.rs cover, with the mortar binary as a child process in a
//! request folder made by the landing verbs in a throwaway repository:
//! - a first save writes a commit with the sentence as subject and prints next mr join, exit 0, outcome saved;
//! - a save in attempt folder k moves refs/mortar/trysnap/<zone>/<request>/<k>, not the request's snapshot ref;
//! - a repeat with nothing changed is exit 0, outcome unchanged, and writes no commit and moves no branch;
//! - a blank or two-line sentence is exit 3 and nothing runs (no lock file, no snapshot ref move);
//! - a new file over the config's cap_bytes is listed with its size and the cap, exit 0;
//! - with a formatter configured the saved tree holds the formatted file;
//! - a formatter that moves the request branch: the save's parent is the formatter's commit
//!   (saves.rs::save_uses_request_branch_tip_moved_by_formatter_as_parent);
//! - a detached HEAD with a formatter is refused naming the moved snapshot ref, which holds the edit, and the
//!   request branch does not move
//!   (saves.rs::detached_head_save_with_formatter_preserves_edit_in_snapshot_without_moving_request_branch);
//! - a formatter past its timeout is listed and the save goes ahead;
//! - a held join-<zone> lock is exit 4 naming the holder;
//! - run in a zone folder it is refused, exit 3;
//! - without a config it is refused, exit 3, next mr init;
//! - --json gives one object with finished_move_from, finished_move_to, commit, change_id, written, refused and cap;
//! - a kill after a join's rebase op and before its folder move, then mr save: the save finishes the move (the
//!   folder at the rebased tip, its work kept in the request's snapshot ref), then saves, naming the finished move;
//! - an undo of a join rebase whose folder move was refused (an edit in the way) leaves the move pending; once the
//!   edit is moved aside, mr save finishes it (the folder back at the tip before the rebase), then saves;
//! - a pending move refused again (the edit still in the way) is exit 3 listing the paths, and nothing is saved.

use oplog::KillPidProbe;
use snapshot::save_with_snapshot;

use super::{save_report, save_run_for, save_sentence, save_step_failure};
use crate::join::{finish_pending_move, moved_before, save_from_formatted, snapshot_moved};
use crate::lock::locked_act;
use crate::report::Report;
use crate::shell::ShellContext;

/// Checks the sentence before any lock, formatter or git; then reads the run (save_run_for: the folder's own
/// branch, its snapshot target, the config's settings, the lock from the verb table's LockNeed and the key). Under
/// `join-<zone>` with recover run first, it first finishes the folder's pending move, a join rebase's, a
/// joined partner request's or an undo's (finish_pending_move, as join does; a refused move is that report and nothing
/// is saved), then saves with the config's excludes and cap. With a formatter configured it runs snapshot_formatted then
/// save_from_snapshot through join's save_from_formatted, so the folder is snapshotted once; without one it commits
/// with snapshot::save_with_snapshot, which takes its own snapshot. Every error of the pending move's snapshot and of
/// the save's steps goes through save_step_failure, with moved_before's line for the snapshot ref when a step before
/// the failed one moved it (none for the pending move's snapshot, the first step). It reports the finished move with
/// the save, and moved_before's line when the pending move's snapshot, the formatter's or save_with_snapshot's own
/// snapshot step moved the snapshot ref (snapshot_moved). The key (--key, else the verb and the folder with no
/// moves) names the lock's holder only and is not passed to oplog's apply; a plain repeat
/// is safe because save writes no commit and moves no branch when the folder equals HEAD, and a
/// finished move is a no-op when repeated. The shell gives an agent verb its folder (actor_for refuses a zone
/// folder, so the record names a request) and a verb that needs the config its config; a missing one is a failure.
pub fn run_save(context: &ShellContext) -> Report {
    let sentence = match save_sentence(&context.args.rest) {
        Ok(sentence) => sentence,
        Err(report) => return report,
    };
    let run = match save_run_for(context, "save") {
        Ok(run) => run,
        Err(report) => return report,
    };
    let (git, toplevel, target, settings) = (&run.git, &run.folder.toplevel, &run.target, &run.settings);
    let moved_line = |snapshot_written: bool| moved_before(false, snapshot_written, &run.request_ref);
    locked_act(git, &run.folder.common, &run.lock, &run.key, &KillPidProbe, "save", || {
        let (finished, finish_wrote) = finish_pending_move(&run, "save", &|error| save_step_failure(&error, None))?;
        let (saved, formatted, earlier) = match &settings.formatter {
            Some(formatter) => {
                let (saved, formatted, written) = save_from_formatted(
                    git,
                    toplevel,
                    target,
                    sentence,
                    &settings.excludes,
                    settings.cap_bytes,
                    Some(formatter),
                )
                .map_err(|(error, written)| save_step_failure(&error, moved_line(finish_wrote || written).as_deref()))?;
                (saved, formatted, finish_wrote || written)
            }
            None => {
                let saved = save_with_snapshot(git, toplevel, target, sentence, &settings.excludes, settings.cap_bytes)
                    .map_err(|error| save_step_failure(&error, moved_line(finish_wrote).as_deref()))?;
                (saved, None, finish_wrote)
            }
        };
        let moved = moved_line(snapshot_moved(earlier, &saved));
        Ok(save_report(&saved.outcome, formatted.as_ref(), finished.as_ref(), moved.as_deref()))
    })
    .unwrap_or_else(|report| report)
}
