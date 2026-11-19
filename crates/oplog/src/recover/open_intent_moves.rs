//! The planned moves of one op while its intent is open.
//! open_intent_moves_tests covers it: open_intents is a pure filter, so they are unit tests.

use super::open_intents;
use crate::store::{OpRecord, RefMove};

/// The moves of `op_id`'s intent when it is open (an Intent record with no later Done or Failed record for
/// that op_id), else None. The guard hook allows a change to the origin or integration branch only when it is
/// listed here with the same ref, old and new; moves of other refs are not checked against this list.
pub(crate) fn open_intent_moves(records: &[OpRecord], op_id: &str) -> Option<Vec<RefMove>> {
    open_intents(records)
        .into_iter()
        .rev()
        .find(|r| r.op_id == op_id)
        .map(|r| r.moves.clone())
}
