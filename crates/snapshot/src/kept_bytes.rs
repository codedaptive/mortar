//! kept_bytes: a copy of the formatter stderr kept so far, read through a lock that may be poisoned.

use std::sync::{Mutex, PoisonError};

/// A copy of the stderr kept so far. A poisoned lock still holds whole chunks, so its bytes are used as they are.
pub(crate) fn kept_bytes(kept: &Mutex<Vec<u8>>) -> Vec<u8> {
    kept.lock().unwrap_or_else(PoisonError::into_inner).clone()
}
