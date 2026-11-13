//! What a caller asks apply to do.
//! A bare type; intent_record_tests exercises it.

use super::RefMove;

/// The caller supplies `op_id` and `at` (unix seconds), so the functions that build records stay pure.
/// `key` makes a repeat safe: a done record with the same key means the op already ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    pub op_id: String,
    pub key: String,
    pub kind: String,
    pub actor: String,
    pub reason: String,
    pub at: u64,
    pub moves: Vec<RefMove>,
}
