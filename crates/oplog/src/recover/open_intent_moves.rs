//! The planned moves of one op while its intent is open.
//! No sibling unit test: it runs open_intents, so its tests are in the integration group:
//! crates/oplog/tests/oplog/open_intent_moves.rs (#1052).

use super::open_intents;
use crate::store::{OpRecord, RefMove};

/// The moves of `op_id`'s intent when it is open (an Intent record with no later Done or Failed record for
/// that op_id), else None. The guard hook (S-60) allows a ref move only when it is listed here with the same
/// ref, old and new.
pub fn open_intent_moves(records: &[OpRecord], op_id: &str) -> Option<Vec<RefMove>> {
    open_intents(records)
        .into_iter()
        .rev()
        .find(|r| r.op_id == op_id)
        .map(|r| r.moves.clone())
}
