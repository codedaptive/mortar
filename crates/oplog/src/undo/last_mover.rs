//! The logged op that last moved a ref after a given op, which undo names when it refuses.

use crate::store::{move_changes, OpPhase, OpRecord};

/// The latest done record at an index above `after` with a move of `refname` that changes the ref; None when
/// no logged op moved it there.
pub fn last_mover<'a>(records: &'a [OpRecord], after: usize, refname: &str) -> Option<&'a OpRecord> {
    records
        .iter()
        .enumerate()
        .rev()
        .take_while(|(i, _)| *i > after)
        .map(|(_, r)| r)
        .find(|r| {
            r.phase == OpPhase::Done
                && r.moves.iter().any(|m| m.refname == refname && move_changes(m))
        })
}
