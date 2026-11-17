//! later_op_refusal: undo never reverses another actor's op. oplog::undo reverses the last done op on the refs it is
//! given, so before calling it run_undo checks that this is still the op choose_undo chose (S-73, S-142).

use oplog::{find_undo_target, OpRecord, OplogError};

use super::undo_error_report;
use crate::report::{is_zero_oid, Report};

/// None when find_undo_target over `records` and `scope` (the refs `records[index]` moved) answers `index`. Else the
/// later done op that moved a ref in `scope` is named: undo_error_report's MovedSince refusal (exit 1) for the
/// first ref in scope it moved, by that op's id and actor, with the value the chosen op left (None for a delete) and
/// the one the later op set.
pub fn later_op_refusal(records: &[OpRecord], index: usize, scope: &[String]) -> Option<Report> {
    let target = records.get(index)?;
    let later = records.get(find_undo_target(records, scope).filter(|&found| found != index)?)?;
    let moved = later.moves.iter().find(|one| scope.contains(&one.refname))?;
    let left = target.moves.iter().find(|one| one.refname == moved.refname).map(|one| &one.new);
    Some(undo_error_report(&OplogError::MovedSince {
        op_id: target.op_id.clone(),
        refname: moved.refname.clone(),
        left: left.filter(|oid| !is_zero_oid(oid)).cloned(),
        actual: Some(moved.new.clone()).filter(|oid| !is_zero_oid(oid)),
        moved_by: Some((later.op_id.clone(), later.actor.clone())),
    }))
}
