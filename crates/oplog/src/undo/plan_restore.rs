//! The moves that set refs back to their values as of an op.

use gitio::Oid;

use super::{value_as_of_op, zero_oid_like};
use crate::store::{OplogError, OpRecord, RefMove};

/// For each ref in `current` (its name and what it holds now), in order, a move from its current value to its
/// value as of `op_id`; a ref already there gets no move, a name listed again is skipped, and a ref absent then
/// is deleted. UnknownOp when no record has `op_id`, even for an empty `current`; otherwise the first refusal
/// of value_as_of_op, in order.
pub fn plan_restore(
    records: &[OpRecord],
    op_id: &str,
    current: &[(String, Option<Oid>)],
) -> Result<Vec<RefMove>, OplogError> {
    if !records.iter().any(|r| r.op_id == op_id) {
        return Err(OplogError::UnknownOp { op_id: op_id.to_string() });
    }
    let mut moves = Vec::new();
    for (i, (refname, cur)) in current.iter().enumerate() {
        if current[..i].iter().any(|(seen, _)| seen == refname) {
            continue;
        }
        let new = match (value_as_of_op(records, op_id, refname)?, cur) {
            (Some(target), Some(c)) if target == *c => continue,
            (None, None) => continue,
            (Some(target), _) => target,
            (None, Some(c)) => zero_oid_like(c)?,
        };
        moves.push(RefMove { refname: refname.clone(), old: cur.clone(), new });
    }
    Ok(moves)
}
