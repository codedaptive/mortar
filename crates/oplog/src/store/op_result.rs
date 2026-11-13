//! What came of an op, as its closing record states it.
//! A bare type; encode_op_record_tests, decode_op_record_tests and decode_op_result_tests exercise it.

use gitio::Oid;

/// JSON `{"outcome": "moved"}`, `{"outcome": "stale", "ref": "...", "actual": "<hex>"|null}` or
/// `{"outcome": "not-moved"}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpResult {
    /// Every ref moved to its new value.
    Moved,
    /// `refname` was not at its expected value; `actual` is what it held (None: the ref was absent).
    Stale {
        refname: String,
        actual: Option<Oid>,
    },
    /// Recovery found every ref at its old value: nothing moved.
    NotMoved,
}
