//! The one check for git's all-zero object id, which as a move's new value deletes the ref; protect, undo,
//! recover and the error display all use it.

use gitio::Oid;

/// True when every hex character is '0': 40 zeros for SHA-1, 64 for SHA-256.
pub(crate) fn oid_is_zero(oid: &Oid) -> bool {
    oid.as_str().bytes().all(|b| b == b'0')
}
