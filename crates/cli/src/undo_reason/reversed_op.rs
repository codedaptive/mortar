//! reversed_op: the op an undo reversed, read back from the reason undo_reason wrote (S-142 (3)).

/// (kind, op_id) from a reason "undo of <kind> op <op_id>", each part non-empty and without whitespace. A reason in
/// any other form, such as one an undo outside mortar wrote, is None.
pub fn reversed_op(reason: &str) -> Option<(String, String)> {
    let (kind, op_id) = reason.strip_prefix("undo of ")?.split_once(" op ")?;
    let word = |part: &str| !part.is_empty() && !part.contains(char::is_whitespace);
    (word(kind) && word(op_id)).then(|| (kind.to_string(), op_id.to_string()))
}
