//! The takeover guards an acquire removed before it refused, read off the refusal so each is logged (S-30).

use super::{LockError, LockRecord};

impl LockError {
    /// The takeover guards removed before this refusal, in order; empty for the variants that carry none.
    pub fn removed_guards(&self) -> &[LockRecord] {
        match self {
            LockError::Held { removed_guards, .. }
            | LockError::TakeoverInProgress { removed_guards, .. }
            | LockError::Contended { removed_guards, .. }
            | LockError::Malformed { removed_guards, .. }
            | LockError::Io { removed_guards, .. } => removed_guards,
            LockError::NotOurs { .. }
            | LockError::BadName(_)
            | LockError::BadRecord(_)
            | LockError::HostUnknown(_) => &[],
        }
    }
}
