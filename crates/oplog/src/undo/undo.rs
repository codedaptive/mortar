//! Undoes the last done op on some refs, as one new op of kind undo through apply_protected.
//! No unit test: it reads the log file and runs git (S-5); find_done_record_tests, find_undo_target_tests,
//! plan_undo_tests, last_mover_tests and caller_op_tests cover its decisions. Nagatha's integration group should
//! cover, in a throwaway repository: undo of a create, an update and a delete; a ref moved since refused as
//! MovedSince naming the mover and its actor, or no mover when the move was made outside mortar; NothingToUndo
//! when no done op moved a ref in scope; a repeat with the same key returns Repeated and moves once; a protected
//! ref refused as Protected on a rewind; the new op logged with kind undo.

use std::path::Path;

use gitio::Git;

use super::{caller_op, find_undo_target, plan_undo, OpCaller};
use crate::apply::{find_done_record, new_op_id, ApplyOutcome};
use crate::protect::{apply_protected, ProtectedRef};
use crate::store::{read_op_log, read_ref_value, unix_now, OplogError};

/// Reverses the last done op with a move of any ref in `scope`, with the refs in `protected` held to
/// fast-forward or creation.
///
/// The caller runs recover first and holds its zone's join lock or the landing lock around the call (S-16), as
/// for apply. A repeat of the caller's done key returns Repeated before anything is planned, so a repeat after
/// the refs moved again still moves them once. An undo is itself an op, so undoing twice redoes.
pub fn undo(
    git: &Git,
    common: &Path,
    scope: &[String],
    caller: &OpCaller,
    protected: &[ProtectedRef],
) -> Result<ApplyOutcome, OplogError> {
    let records = read_op_log(common)?.records;
    if let Some(done) = find_done_record(&records, &caller.key) {
        return Ok(ApplyOutcome::Repeated(done.clone()));
    }
    let target = find_undo_target(&records, scope)
        .ok_or_else(|| OplogError::NothingToUndo { refs: scope.to_vec() })?;
    let current = records[target]
        .moves
        .iter()
        .map(|m| Ok((m.refname.clone(), read_ref_value(git, &m.refname)?)))
        .collect::<Result<Vec<_>, OplogError>>()?;
    let moves = plan_undo(&records, target, &current)?;
    let op = caller_op(caller, "undo", new_op_id(), unix_now(), moves);
    apply_protected(git, common, &op, protected)
}
