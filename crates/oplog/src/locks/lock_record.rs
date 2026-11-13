//! What a lock file holds: who took the lock, on which host, for which op, since when.
//! A bare type; encode_lock_record_tests and decode_lock_record_tests exercise it.

/// The contents of one lock file. `since` is in unix seconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockRecord {
    pub pid: u32,
    pub host: String,
    pub key: String,
    pub since: u64,
}
