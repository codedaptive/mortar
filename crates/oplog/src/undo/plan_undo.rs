//! The moves that reverse one done op, refused when a ref it moved has moved since.

use gitio::Oid;

use super::{last_mover, value_after_move, zero_oid_like};
use crate::store::{move_changes, OplogError, OpRecord, RefMove};

/// For each move of `records[target]` that changes its ref, in order: a move from what the op left back to
/// what the ref held before it (a create reverses to a delete, a delete to a create).
///
/// `current` pairs each ref name with what it holds now; a ref not listed counts as absent. The first ref
/// whose value is not what the op left returns MovedSince, naming the latest later done op that moved it, or
/// none when no logged op did. `target` is an index into `records`, as find_undo_target gives; an index out of
/// range panics.
pub fn plan_undo(
    records: &[OpRecord],
    target: usize,
    current: &[(String, Option<Oid>)],
) -> Result<Vec<RefMove>, OplogError> {
    let op = &records[target];
    let mut moves = Vec::new();
    for m in op.moves.iter().filter(|m| move_changes(m)) {
        let left = value_after_move(&m.new);
        let actual = current
            .iter()
            .find(|(name, _)| *name == m.refname)
            .and_then(|(_, value)| value.clone());
        if actual != left {
            return Err(OplogError::MovedSince {
                op_id: op.op_id.clone(),
                refname: m.refname.clone(),
                left,
                actual,
                moved_by: last_mover(records, target, &m.refname)
                    .map(|r| (r.op_id.clone(), r.actor.clone())),
            });
        }
        moves.push(RefMove {
            refname: m.refname.clone(),
            old: left,
            new: match &m.old {
                Some(old) => old.clone(),
                None => zero_oid_like(&m.new)?,
            },
        });
    }
    Ok(moves)
}
