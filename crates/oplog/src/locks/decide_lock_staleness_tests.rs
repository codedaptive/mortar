//! The staleness decision: only a dead holder on this host is stale.

use super::{decide_lock_staleness, LockRecord, LockStaleness};

fn holder_on(host: &str) -> LockRecord {
    LockRecord {
        pid: 4242,
        host: host.to_string(),
        key: "land".to_string(),
        since: 1763030400,
    }
}

#[test]
fn same_host_dead_pid_is_stale() {
    assert_eq!(
        decide_lock_staleness(&holder_on("build-mac"), "build-mac", false),
        LockStaleness::Stale
    );
}

#[test]
fn other_host_dead_pid_is_held() {
    assert_eq!(
        decide_lock_staleness(&holder_on("other-box"), "build-mac", false),
        LockStaleness::Held
    );
}

#[test]
fn same_host_live_pid_is_held() {
    assert_eq!(
        decide_lock_staleness(&holder_on("build-mac"), "build-mac", true),
        LockStaleness::Held
    );
}

#[test]
fn other_host_live_pid_is_held() {
    assert_eq!(
        decide_lock_staleness(&holder_on("other-box"), "build-mac", true),
        LockStaleness::Held
    );
}
