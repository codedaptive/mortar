//! One planned ref move in an op: the ref, the value it must hold first, the value to set.
//! A bare type; encode_op_record_tests, decode_op_record_tests and decode_ref_move_tests exercise it.

use gitio::Oid;

/// JSON `{"ref": "...", "old": "<hex>"|null, "new": "<hex>"}`. `old` None means the ref must not exist.
/// An all-zero `new` deletes the ref; it is carried as given, never refused here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefMove {
    pub refname: String,
    pub old: Option<Oid>,
    pub new: Oid,
}
