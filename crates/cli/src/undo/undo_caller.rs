//! undo_caller: who asks oplog for an undo, and why.

use oplog::{OpCaller, OpRecord};

use crate::undo_reason::undo_reason;

/// Key "undo:<op_id>" of `target`, so a repeat of the same undo is oplog's Repeated and never a redo; `actor`, the
/// folder's; reason undo_reason's "undo of <kind> op <op_id>", which choose_undo and find_pending_move read back.
pub fn undo_caller(target: &OpRecord, actor: &str) -> OpCaller {
    OpCaller {
        key: format!("undo:{}", target.op_id),
        actor: actor.to_string(),
        reason: undo_reason(&target.kind, &target.op_id),
    }
}
