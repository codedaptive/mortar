//! How a lock refusal reads: every refusal names its holder, path, zone or reason, and says what to do.

use std::fmt;

use super::{LockError, LockRecord};

/// "pid P on host H for op K since S (unix seconds)": the holder as each refusal over a held lock names it.
fn holder_text(holder: &LockRecord) -> String {
    format!(
        "pid {} on host {} for op {} since {} (unix seconds)",
        holder.pid, holder.host, holder.key, holder.since
    )
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockError::Held { path, holder } => write!(
                f,
                "{} is held by {}; wait for it to finish",
                path.display(),
                holder_text(holder)
            ),
            LockError::TakeoverInProgress { path, holder } => write!(
                f,
                "takeover guard {} is held by {}: a takeover is running or crashed; \
                 if no takeover is running, remove {} by hand",
                path.display(),
                holder_text(holder),
                path.display()
            ),
            LockError::NotOurs {
                path,
                holder: Some(holder),
            } => write!(
                f,
                "{} is not ours to release: it is held by {}; leave it, its holder releases it",
                path.display(),
                holder_text(holder)
            ),
            LockError::NotOurs { path, holder: None } => {
                write!(f, "{} is not ours to release: no lock file, so nothing to release", path.display())
            }
            LockError::Contended { path } => write!(
                f,
                "{} changed hands repeatedly while acquiring; try again",
                path.display()
            ),
            LockError::BadName(zone) => write!(
                f,
                "bad lock zone name {zone:?}: use 1 to 100 ASCII letters, digits, '-', '_' or '.', \
                 not starting with '.'"
            ),
            LockError::BadRecord(reason) => write!(f, "lock record refused: {reason}; fix the host or op key given to the lock"),
            LockError::Malformed {
                path: Some(path),
                reason,
            } => write!(
                f,
                "{} is not a lock record ({reason}); check no mortar process holds it, then remove it by hand",
                path.display()
            ),
            LockError::Malformed { path: None, reason } => {
                write!(f, "lock record is malformed: {reason}; mortar did not write this text, so check what did")
            }
            LockError::HostUnknown(reason) => write!(
                f,
                "cannot read this host's name: {reason}; check that `hostname` runs on this machine"
            ),
            LockError::Io { path, source } => write!(
                f,
                "{}: {source}; check that path's folder and its permissions, then run again",
                path.display()
            ),
        }
    }
}
