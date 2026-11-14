//! One line of the op log.
//! A bare type; encode_op_record_tests and decode_op_record_tests exercise its JSON form.

use super::{OpPhase, OpResult, RefMove};

/// JSON keys in order: op_id, key, kind, actor, reason, at, phase, moves, result (omitted when None);
/// a line written with values adds a `values` key (ValuedRecord); a marked line adds a last `remote` key
/// (MarkedRecord).
/// `at` is in unix seconds. An intent and its closing record share op_id, key, kind, actor, reason and moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpRecord {
    pub op_id: String,
    pub key: String,
    pub kind: String,
    pub actor: String,
    pub reason: String,
    pub at: u64,
    pub phase: OpPhase,
    pub moves: Vec<RefMove>,
    pub result: Option<OpResult>,
}
