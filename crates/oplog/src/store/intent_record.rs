//! The record that announces an op before its ref transaction.

use super::{Op, OpPhase, OpRecord};

/// The op's fields and every planned move, in phase Intent with no result.
pub fn intent_record(op: &Op) -> OpRecord {
    OpRecord {
        op_id: op.op_id.clone(),
        key: op.key.clone(),
        kind: op.kind.clone(),
        actor: op.actor.clone(),
        reason: op.reason.clone(),
        at: op.at,
        phase: OpPhase::Intent,
        moves: op.moves.clone(),
        result: None,
    }
}
