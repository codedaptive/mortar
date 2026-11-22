//! reverse_and_step: run_undo's undo of one chosen step: the snapshot (whose refusal in an attempt folder with an
//! unborn HEAD names the try branch, attempt_head_next), the check against a later op, the oplog undo and the folder
//! step, with the paused join put back for a resolve step.
//! No unit test: it runs git, snapshot or the op log; its decisions have their tests.

use folders::FolderKind;
use oplog::{undo, ApplyOutcome, OpRecord, ValuedRecord};
use snapshot::{snapshot, SnapshotError};

use super::folder_step::folder_step;
use super::{
    already_undone_report, failed_undo_report, later_op_refusal, undo_caller, undo_error_report, undo_report,
    with_restored, UndoStep,
};
use crate::join::RequestRun;
use crate::report::{attempt_head_next, Report};
use crate::save::snapshot_error_report;
use crate::shell::ShellContext;

const VERB: &str = "undo";

/// Snapshots the folder first (a SnapshotError is snapshot_error_report relabelled undo; in an attempt folder, whose
/// branch is run.request_ref, an unborn HEAD's next is attempt_head_next's, naming the try branch, since the
/// request's branch is not the attempt's; its lines, exit, outcome and owner stay), checks that the op at
/// `index` is still the last on its refs (later_op_refusal), undoes it through oplog on exactly its refs, then makes
/// the folder step (folder_step): SetIndex sets the index to the tree of the commit the branch now holds; MoveFolder
/// finishes the undo's own folder move (a refused move is that report and stays pending); RestoreJoin puts the
/// paused join back, then sets the index (a refused restore is restore_failed_report, and a repeated undo finishes
/// it). Done is undo_report, Repeated already_undone_report, the folder step made again; either is with_restored
/// when a restore ran. Failed, a ref moved during the call outside mortar, is refused naming it. A refusal after
/// the snapshot (a later op, Failed) keeps the snapshot.
pub(super) fn reverse_and_step(
    context: &ShellContext,
    run: &RequestRun,
    records: &[ValuedRecord],
    index: usize,
    step: UndoStep,
) -> Result<Report, Report> {
    let (excludes, cap) = (&run.settings.excludes, run.settings.cap_bytes);
    let kept = snapshot(&run.git, &run.folder.toplevel, &run.target, excludes, cap)
        .map_err(|error| snapshot_refusal(run, &error))?;
    let plain: Vec<OpRecord> = records.iter().map(|valued| valued.record.clone()).collect();
    let target = &plain[index];
    let scope: Vec<String> = target.moves.iter().map(|one| one.refname.clone()).collect();
    if let Some(refusal) = later_op_refusal(&plain, index, &scope) {
        return Err(refusal);
    }
    let caller = undo_caller(target, &context.actor);
    let outcome = undo(&run.git, &run.folder.common, &scope, &caller, &context.protected)
        .map_err(|error| undo_error_report(&error))?;
    let (report, undo_op) = match outcome {
        ApplyOutcome::Done(done) => (undo_report(target, &done.op_id, &kept.commit, step), done.op_id),
        ApplyOutcome::Repeated(done) => (already_undone_report(&done.op_id, &target.op_id, step), done.op_id),
        ApplyOutcome::Failed(failed) => return Err(failed_undo_report(&target.op_id, &failed)),
    };
    let restored = folder_step(run, step, &records[index], &undo_op)?;
    Ok(restored.iter().fold(report, with_restored))
}

/// snapshot_error_report for `error` relabelled undo, its next attempt_head_next's when that gives one: the folder's
/// branch is the try branch only in an attempt folder.
fn snapshot_refusal(run: &RequestRun, error: &SnapshotError) -> Report {
    let report = Report { verb: VERB.to_string(), ..snapshot_error_report(error) };
    let attempt_branch = match run.folder.record.kind {
        FolderKind::Attempt(_) => run.request_ref.strip_prefix("refs/heads/"),
        FolderKind::Request | FolderKind::Zone => None,
    };
    match attempt_head_next(error, attempt_branch) {
        Some(next) => Report { next, ..report },
        None => report,
    }
}
