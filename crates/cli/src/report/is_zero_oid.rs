//! The all-zero object id, which as a move's new value deletes the ref (S-9).
//! oplog keeps its own check crate-private, so the cli's verbs (log, restore) share this one.

use gitio::Oid;

/// True when every hex digit is '0': 40 zeros for SHA-1, 64 for SHA-256.
pub fn is_zero_oid(oid: &Oid) -> bool {
    oid.as_str().bytes().all(|b| b == b'0')
}
