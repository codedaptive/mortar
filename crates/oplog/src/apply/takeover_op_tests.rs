//! takeover_op: no op without a takeover; with one, the op names the new holder, the lock and the old holder.

use std::path::PathBuf;

use super::takeover_op;
use crate::locks::{AcquiredLock, LockRecord};
use crate::store::Op;

fn lock(took_over: Option<LockRecord>) -> AcquiredLock {
    AcquiredLock {
        path: PathBuf::from("/repo/.git/mortar/locks/landing.lock"),
        record: LockRecord {
            pid: 4321,
            host: "mac-b".to_string(),
            key: "land:zone-b".to_string(),
            since: 1763030500,
        },
        took_over,
    }
}

#[test]
fn no_takeover_gives_no_op() {
    assert_eq!(takeover_op(&lock(None), "op-9".to_string(), 1763030501), None);
}

#[test]
fn a_takeover_gives_the_logged_op() {
    let old = LockRecord {
        pid: 1234,
        host: "mac-a".to_string(),
        key: "land:zone-a".to_string(),
        since: 1763030400,
    };
    assert_eq!(
        takeover_op(&lock(Some(old)), "op-9".to_string(), 1763030501),
        Some(Op {
            op_id: "op-9".to_string(),
            key: "lock-takeover:/repo/.git/mortar/locks/landing.lock:1234@mac-a:1763030400".to_string(),
            kind: "lock-takeover".to_string(),
            actor: "4321@mac-b".to_string(),
            reason: "took over the stale lock /repo/.git/mortar/locks/landing.lock from pid 1234 on host \
                     mac-a, held for key land:zone-a"
                .to_string(),
            at: 1763030501,
            moves: Vec::new(),
        })
    );
}
