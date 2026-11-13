//! Whether an existing lock may be taken over, from its holder and facts gathered by the caller.

use super::{LockRecord, LockStaleness};

/// Stale only when the holder ran on this host and its pid is dead. A holder on another host is
/// always held: this host cannot see whether that process is alive.
pub fn decide_lock_staleness(holder: &LockRecord, this_host: &str, pid_alive: bool) -> LockStaleness {
    if holder.host == this_host && !pid_alive {
        LockStaleness::Stale
    } else {
        LockStaleness::Held
    }
}
