//! What a ref held as of an op, read from the done records around it.

use gitio::Oid;

use super::value_after_move;
use crate::store::{OpPhase, OplogError, OpRecord, RefMove};

/// The pivot is the last record with id `op_id`. The value is what the last done move of `refname` at or
/// before the pivot left; failing that, what the first done move of it after the pivot found (its `old`).
/// Ok(None) means the ref was absent then. UnknownOp when no record has `op_id`; ValueUnknown when no done
/// record moves `refname`.
pub(crate) fn value_as_of_op(
    records: &[OpRecord],
    op_id: &str,
    refname: &str,
) -> Result<Option<Oid>, OplogError> {
    let pivot = records
        .iter()
        .rposition(|r| r.op_id == op_id)
        .ok_or_else(|| OplogError::UnknownOp { op_id: op_id.to_string() })?;
    let (upto, after) = records.split_at(pivot + 1);
    if let Some(m) = done_moves(upto, refname).last() {
        return Ok(value_after_move(&m.new));
    }
    match done_moves(after, refname).next() {
        Some(m) => Ok(m.old.clone()),
        None => Err(OplogError::ValueUnknown {
            op_id: op_id.to_string(),
            refname: refname.to_string(),
        }),
    }
}

/// The moves of `refname` in the done records of `records`, in log order.
fn done_moves<'a>(records: &'a [OpRecord], refname: &'a str) -> impl Iterator<Item = &'a RefMove> {
    records
        .iter()
        .filter(|r| r.phase == OpPhase::Done)
        .flat_map(|r| r.moves.iter())
        .filter(move |m| m.refname == refname)
}
