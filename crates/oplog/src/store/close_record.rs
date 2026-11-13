//! The record that closes an intent.

use super::{OpPhase, OpRecord, OpResult};

/// The intent's op_id, key, kind, actor, reason and moves, with the given phase, result and time.
pub fn close_record(intent: &OpRecord, phase: OpPhase, result: OpResult, at: u64) -> OpRecord {
    OpRecord {
        at,
        phase,
        result: Some(result),
        ..intent.clone()
    }
}
