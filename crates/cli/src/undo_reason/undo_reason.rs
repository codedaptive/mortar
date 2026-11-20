//! undo_reason: the reason an undo op records, naming the op it reversed.

/// "undo of <kind> op <op_id>": the reversed op's kind and id, which reversed_op reads back.
pub fn undo_reason(kind: &str, op_id: &str) -> String {
    format!("undo of {kind} op {op_id}")
}
