//! run_undo: `mr undo [--key <key>] [--json]`, the agent verb that undoes its folder's own last step: a save, a
//! join's rebase or a resolve's final rebase, whose paused join it puts back.
//! No unit test: it takes the lock, snapshots, runs git and the op log; own_refs, choose_undo, undo_refusal,
//! later_op_refusal, undo_caller, undo_report, already_undone_report, nothing_to_undo_report, failed_undo_report,
//! undo_error_report, restore_check_report, restore_failed_report and with_restored are its decisions and have their
//! tests.
//! The integration tests crates/cli/tests/across_kits/undo_save_join_resolve/*.rs cover, with the mortar binary as a
//! child process in a request folder made by the landing verbs in a throwaway repository:
//! - save then undo keeps the files and shows them unsaved (the index at the branch's restored tree);
//! - undo after a join's rebase moves the folder back to the tip before the rebase; with an edit in the way the move
//!   is refused and left pending, status gives next mr undo, and the next undo, save or join finishes it;
//! - undo of a joined request (the zone branch moved) is refused, exit 1;
//! - a ref moved since by another actor is refused, exit 1, naming the op and the actor;
//! - nothing to undo is exit 0, its next as status would give;
//! - undo of a resolve step restores the paused join: status shows the join paused, next mr resolve, and mr resolve
//!   then answers with the same folder_tree; a repeat answers already-done naming the undo op;
//! - each pre-check refusal is exit 1 with nothing moved: another paused join's mortar-join.json present
//!   (StatePresent), and a mortar-join.json that does not decode (StateCorrupt), the file kept;
//! - undo of a resolve recorded without its paused join (no join_state value) is refused, exit 1, nothing changed;
//! - `mr log --json` shows the join_state value on the resolve op's line;
//! - a resolve undo whose own folder move is pending with a file in the way is the move refusal, exit 1, the
//!   snapshot kept and no undo op (crates/cli/tests/across_kits/undo_save_join_resolve/refusals.rs covers it);
//! - the snapshot taken first is restorable with mr restore;
//! - a repeat after a kill answers already-done, finishing the folder step the kill left;
//! - a held join-<zone> lock is exit 3 naming the holder;
//! - undo in an attempt folder whose HEAD is unborn is refused, exit 1, owner the folder's HEAD, its next checking out
//!   the try branch, with no undo op and no ref moved;
//! - in attempt folder k, only attempt k's own save is undone, never the request's, and the reverse.

use folders::FolderKind;
use oplog::{read_valued_log, KillPidProbe, ValuedRecord};

use super::folder_step::folder_step;
use super::reverse_and_step::reverse_and_step;
use super::set_index_to_branch::set_index_to_branch;
use super::{
    already_undone_report, choose_undo, nothing_to_undo_report, own_refs, restore_check_report, undo_error_report,
    undo_refusal, with_restored, UndoChoice, UndoStep,
};
use crate::args::no_arguments_report;
use crate::join::{finish_pending_move, snapshot_failure, RequestRun};
use crate::lock::locked_act;
use crate::report::{failure_report, Report};
use crate::save::save_run_for;
use crate::shell::ShellContext;
use crate::status::{attempt_next, read_status, status_next};
use crate::undo_reason::done_record;

const VERB: &str = "undo";

/// Any argument is no_arguments_report's usage report. The run is save_run_for's for undo (the folder's own branch,
/// its snapshot target, the lock join-<zone> and the key, which names the lock's holder only; oplog gets
/// "undo:<op id>"). Under the lock, with recover run first: choose_undo over the op log (a read
/// only); a refusal or nothing to undo changes nothing, a pending move left as it is. For a RestoreJoin step,
/// join's check_restore_join_state then runs (a read only), and its refusal is restore_check_report with nothing
/// moved. Otherwise the pending move is finished first, then the step is undone (reverse_and_step);
/// a refusal after the move is finished and the snapshot taken (a later op, a ref moved during the call) keeps the
/// snapshot. Already undone, the folder step is made again (already_undone).
pub fn run_undo(context: &ShellContext) -> Report {
    let args = &context.args;
    if let Some(usage) = no_arguments_report(VERB, &args.rest, args.json) {
        return usage;
    }
    let run = match save_run_for(context, VERB) {
        Ok(run) => run,
        Err(report) => return report,
    };
    let Some(own) = own_refs(&run.folder.record) else {
        return failure_report(VERB, "this folder's record names no branch of its own; actor_for refuses a zone folder");
    };
    locked_act(&run.git, &run.folder.common, &run.lock, &run.key, &KillPidProbe, VERB, || {
        let records = read_valued_log(&run.folder.common).map_err(|error| undo_error_report(&error))?.records;
        let choice = choose_undo(&records, &context.actor, &own, &format!("refs/heads/zone/{}", run.target.zone));
        match choice {
            UndoChoice::Nothing => Ok(nothing_to_undo_report(&next_as_status(context, &run))),
            UndoChoice::ZoneOwns { .. } | UndoChoice::Resolved { .. } | UndoChoice::NotAStep { .. } => {
                undo_refusal(&choice).ok_or_else(|| failure_report(VERB, "a refused choice gave no refusal"))
            }
            UndoChoice::Step { index, step } => {
                if step == UndoStep::RestoreJoin {
                    ::join::check_restore_join_state(&run.git, &run.folder.toplevel, &records[index])
                        .map_err(|error| restore_check_report(&error))?;
                }
                finish_pending_move(&run, VERB, &|error| snapshot_failure(VERB, error))?;
                reverse_and_step(context, &run, &records, index, step)
            }
            UndoChoice::AlreadyUndone { undo_op, reversed_op, step } => {
                already_undone(&run, &records, &undo_op, &reversed_op, step)
            }
        }
    })
    .unwrap_or_else(|report| report)
}

/// The pending move finished, then the folder step made again: SetIndex re-sets the index to the branch's tree;
/// MoveFolder needs nothing more; RestoreJoin reads the reversed op's Done record and runs folder_step, which puts
/// the folder record and the paused join back and then sets the index (a record missing from the log is
/// failure_report). The report is already_undone_report, with_restored when a restore ran.
fn already_undone(
    run: &RequestRun,
    records: &[ValuedRecord],
    undo_op: &str,
    reversed_op: &str,
    step: UndoStep,
) -> Result<Report, Report> {
    finish_pending_move(run, VERB, &|error| snapshot_failure(VERB, error))?;
    let restored = match step {
        UndoStep::SetIndex => set_index_to_branch(run).map(|()| None)?,
        UndoStep::MoveFolder => None,
        UndoStep::RestoreJoin => {
            let Some(target) = done_record(records, reversed_op) else {
                let why = format!(
                    "op {reversed_op}, which undo op {undo_op} reversed, is not in the op log, so the paused join \
                     cannot be read"
                );
                return Err(failure_report(VERB, &why));
            };
            folder_step(run, step, target, undo_op)?
        }
    };
    Ok(restored.iter().fold(already_undone_report(undo_op, reversed_op, step), with_restored))
}

/// What status would give as next for this folder (attempt_next in an attempt folder); "mr status" when it
/// cannot be read.
fn next_as_status(context: &ShellContext, run: &RequestRun) -> String {
    match (read_status(run.folder, &run.settings, &context.actor, VERB), run.folder.record.kind) {
        (Ok(state), FolderKind::Attempt(_)) => attempt_next(&state),
        (Ok(state), FolderKind::Request | FolderKind::Zone) => status_next(&state),
        (Err(_), _) => "mr status".to_string(),
    }
}
