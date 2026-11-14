//! The op an undo or a restore hands to apply_protected.

use super::OpCaller;
use crate::store::{Op, RefMove};

/// The caller's key, actor and reason with `kind`, `op_id`, `at` (unix seconds) and `moves`, all as given.
pub fn caller_op(caller: &OpCaller, kind: &str, op_id: String, at: u64, moves: Vec<RefMove>) -> Op {
    Op {
        op_id,
        key: caller.key.clone(),
        kind: kind.to_string(),
        actor: caller.actor.clone(),
        reason: caller.reason.clone(),
        at,
        moves,
    }
}
