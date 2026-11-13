//! The verdict on an existing lock: stale (may be taken over) or held.
//! A bare type; decide_lock_staleness_tests exercises it.

/// Whether an existing lock may be taken over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStaleness {
    Stale,
    Held,
}
