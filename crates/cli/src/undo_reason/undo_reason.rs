//! undo_reason: the reason an undo op records, naming the op it reversed (S-142 (3)).

/// "undo of <kind> op <op_id>": the reversed op's kind and id, which reversed_op reads back.
pub fn undo_reason(kind: &str, op_id: &str) -> String {
    format!("undo of {kind} op {op_id}")
}
