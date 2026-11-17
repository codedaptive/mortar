//! run_undo: `mr undo [--key <key>] [--json]`, the agent verb that undoes its folder's own last step: a save or
//! a join's rebase (S-73, S-142, S-143).
//! No unit test (S-104): it takes the lock, snapshots, runs git and the op log; own_refs, choose_undo, undo_refusal,
//! later_op_refusal, undo_caller, undo_report, already_undone_report, nothing_to_undo_report, failed_undo_report and
//! undo_error_report are its decisions and have their tests.
//! Nagatha's integration group (RITs) is to cover, with the mortar binary as a child process in a request folder
//! made by the landing verbs in a throwaway repository:
//! - save then undo keeps the files and shows them unsaved (the index at the branch's restored tree);
//! - undo after a join's rebase moves the folder back to the tip before the rebase; with an edit in the way the move
//!   is refused and left pending, status gives next mr undo, and the next undo, save or join finishes it;
//! - undo of a joined request (the zone branch moved) is refused, exit 1;
//! - a ref moved since by another actor is refused, exit 1, naming the op and the actor;
//! - nothing to undo is exit 0, its next as status would give;
//! - undo of a resolve step is refused, exit 1, nothing changed;
//! - the snapshot taken first is restorable with mr restore;
//! - a repeat after a kill answers already-done, finishing the folder step the kill left;
//! - a held join-<zone> lock is exit 3 naming the holder;
//! - in attempt folder k, only attempt k's own save is undone, never the request's, and the reverse.

use folders::FolderKind;
use oplog::{read_valued_log, undo, ApplyOutcome, KillPidProbe, OpRecord, ValuedRecord};
use snapshot::snapshot;

use super::{
    already_undone_report, choose_undo, failed_undo_report, later_op_refusal, nothing_to_undo_report, own_refs,
    undo_caller, undo_error_report, undo_refusal, undo_report, UndoChoice, UndoStep,
};
use crate::args::no_arguments_report;
use crate::join::{finish_pending_move, RequestRun};
use crate::lock::locked_act;
use crate::report::{failure_report, Report};
use crate::save::{save_run_for, snapshot_error_report};
use crate::shell::ShellContext;
use crate::status::{attempt_next, read_status, status_next};

const VERB: &str = "undo";

/// Any argument is no_arguments_report's usage report. The run is save_run_for's for undo (the folder's own branch,
/// its snapshot target, the lock join-<zone> and the key, which names the lock's holder only; oplog gets
/// "undo:<op id>", S-142 (3)). Under the lock, with recover run first (S-16): choose_undo over the op log (a read
/// only); a refusal or nothing to undo changes nothing, a pending move left as it is (S-143). Otherwise the
/// pending move is finished first (S-143), then the step is undone (undo_step), or, already undone, the folder step
/// is made again: SetIndex re-sets the index to the branch's tree; MoveFolder needs nothing more.
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
                finish_pending_move(&run, VERB)?;
                undo_step(context, &run, &records, index, step)
            }
            UndoChoice::AlreadyUndone { undo_op, reversed_op, step } => {
                finish_pending_move(&run, VERB)?;
                match step {
                    UndoStep::SetIndex => set_index_to_branch(&run)?,
                    UndoStep::MoveFolder => {}
                }
                Ok(already_undone_report(&undo_op, &reversed_op, step))
            }
        }
    })
    .unwrap_or_else(|report| report)
}

/// Snapshots the folder first (a SnapshotError is snapshot_error_report relabelled undo), checks that the op at
/// `index` is still the last on its refs (later_op_refusal), undoes it through oplog on exactly its refs, then makes
/// the folder step: SetIndex sets the index to the tree of the commit the branch now holds; MoveFolder finishes the
/// undo's own folder move (a refused move is that report and stays pending). Repeated is already-done, the folder
/// step made again; Failed, a ref moved during the call outside mortar, is refused naming it.
fn undo_step(
    context: &ShellContext,
    run: &RequestRun,
    records: &[ValuedRecord],
    index: usize,
    step: UndoStep,
) -> Result<Report, Report> {
    let (excludes, cap) = (&run.settings.excludes, run.settings.cap_bytes);
    let kept = snapshot(&run.git, &run.folder.toplevel, &run.target, excludes, cap)
        .map_err(|error| Report { verb: VERB.to_string(), ..snapshot_error_report(&error) })?;
    let plain: Vec<OpRecord> = records.iter().map(|valued| valued.record.clone()).collect();
    let target = &plain[index];
    let scope: Vec<String> = target.moves.iter().map(|one| one.refname.clone()).collect();
    if let Some(refusal) = later_op_refusal(&plain, index, &scope) {
        return Err(refusal);
    }
    let caller = undo_caller(target, &context.actor);
    let outcome = undo(&run.git, &run.folder.common, &scope, &caller, &context.protected)
        .map_err(|error| undo_error_report(&error))?;
    match outcome {
        ApplyOutcome::Done(done) => {
            folder_step(run, step)?;
            Ok(undo_report(target, &done.op_id, &kept.commit, step))
        }
        ApplyOutcome::Repeated(done) => {
            folder_step(run, step)?;
            Ok(already_undone_report(&done.op_id, &target.op_id, step))
        }
        ApplyOutcome::Failed(failed) => Err(failed_undo_report(&target.op_id, &failed)),
    }
}

fn folder_step(run: &RequestRun, step: UndoStep) -> Result<(), Report> {
    match step {
        UndoStep::SetIndex => set_index_to_branch(run),
        UndoStep::MoveFolder => finish_pending_move(run, VERB).map(|_| ()),
    }
}

/// The folder's index set to the tree of the commit its branch holds; its files and HEAD untouched.
fn set_index_to_branch(run: &RequestRun) -> Result<(), Report> {
    let failed = |_| failure_report(VERB, "git failed while setting the folder's index to its branch's tree");
    let Some(commit) = run.git.read_ref(&run.request_ref).map_err(failed)? else {
        return Err(failure_report(VERB, &format!("{} is gone, so the folder's index was not set", run.request_ref)));
    };
    let tree = run.git.commit_info(&commit).map_err(failed)?.tree;
    run.git.set_index(&run.folder.toplevel, &tree).map_err(failed)
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
