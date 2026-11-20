//! failed_undo_report: the report of an undo op oplog answered Failed: a ref was not where the plan read it, so
//! nothing moved. run_undo holds join-<zone>, so the ref moved outside mortar.

use oplog::{OpRecord, OpResult, OplogError};

use super::undo_error_report;
use crate::report::{failure_report, Report};

/// For a Stale result, undo_error_report's MovedSince refusal (exit 1) naming `target_op`, the stale ref and that it
/// moved outside mortar. A Failed record with any other result is failure_report naming the undo op.
pub fn failed_undo_report(target_op: &str, failed: &OpRecord) -> Report {
    match &failed.result {
        Some(OpResult::Stale { refname, actual }) => undo_error_report(&OplogError::MovedSince {
            op_id: target_op.to_string(),
            refname: refname.clone(),
            left: failed.moves.iter().find(|one| one.refname == *refname).and_then(|one| one.old.clone()),
            actual: actual.clone(),
            moved_by: None,
        }),
        Some(OpResult::Moved) | Some(OpResult::NotMoved { .. }) | None => {
            failure_report("undo", &format!("the undo op {} failed and moved nothing", failed.op_id))
        }
    }
}
