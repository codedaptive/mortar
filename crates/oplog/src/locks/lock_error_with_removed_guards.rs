//! Puts an acquire's takeover guard removals on the refusal it returns, so a refusal still names them (S-30).

use super::{LockError, LockRecord};

impl LockError {
    /// This refusal with `removed_guards` in place of the list it carried; a variant that carries no list is
    /// returned unchanged.
    pub(crate) fn with_removed_guards(self, removed_guards: Vec<LockRecord>) -> LockError {
        match self {
            LockError::Held { path, holder, .. } => LockError::Held {
                path,
                holder,
                removed_guards,
            },
            LockError::TakeoverInProgress { path, holder, .. } => LockError::TakeoverInProgress {
                path,
                holder,
                removed_guards,
            },
            LockError::Contended { path, .. } => LockError::Contended { path, removed_guards },
            LockError::Malformed { path, reason, .. } => LockError::Malformed {
                path,
                reason,
                removed_guards,
            },
            LockError::Io { path, source, .. } => LockError::Io {
                path,
                source,
                removed_guards,
            },
            other @ (LockError::NotOurs { .. }
            | LockError::BadName(_)
            | LockError::BadRecord(_)
            | LockError::HostUnknown(_)) => other,
        }
    }
}
