//! guard_removal_op: one op per removed guard, in order, each naming the new holder, the guard and its old holder.

use std::path::{Path, PathBuf};

use super::guard_removal_op;
use crate::locks::{LockError, LockRecord};
use crate::store::Op;

const LOCK: &str = "/repo/.git/mortar/locks/land.lock";

fn me() -> LockRecord {
    LockRecord {
        pid: 4321,
        host: "mac-b".to_string(),
        key: "land:zone-b".to_string(),
        since: 1763030500,
    }
}

fn old_guard(since: u64) -> LockRecord {
    LockRecord {
        pid: 1234,
        host: "mac-b".to_string(),
        key: "land:zone-a".to_string(),
        since,
    }
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("op-{n}")
    }
}

fn expected(op_id: &str, key: &str) -> Op {
    Op {
        op_id: op_id.to_string(),
        key: key.to_string(),
        kind: "takeover-guard-removal".to_string(),
        actor: "4321@mac-b".to_string(),
        reason: "removed the stale takeover guard /repo/.git/mortar/locks/land.lock.takeover left by pid \
                 1234 on host mac-b, held for key land:zone-a"
            .to_string(),
        at: 1763030501,
        moves: Vec::new(),
    }
}

#[test]
fn an_empty_list_gives_no_ops() {
    assert_eq!(guard_removal_op(Path::new(LOCK), &me(), &[], &mut counter(), 1763030501), Vec::new());
}

#[test]
fn one_removal_gives_the_logged_op() {
    let mut next = || "op-9".to_string();
    assert_eq!(
        guard_removal_op(Path::new(LOCK), &me(), &[old_guard(1763030400)], &mut next, 1763030501),
        vec![Op {
            op_id: "op-9".to_string(),
            key: "takeover-guard-removal:/repo/.git/mortar/locks/land.lock.takeover:1234@mac-b:1763030400"
                .to_string(),
            kind: "takeover-guard-removal".to_string(),
            actor: "4321@mac-b".to_string(),
            reason: "removed the stale takeover guard /repo/.git/mortar/locks/land.lock.takeover left by pid \
                     1234 on host mac-b, held for key land:zone-a"
                .to_string(),
            at: 1763030501,
            moves: Vec::new(),
        }]
    );
}

#[test]
fn two_removals_give_two_ops_in_order_with_distinct_keys() {
    let removed = [old_guard(1763030400), old_guard(1763030450)];
    assert_eq!(
        guard_removal_op(Path::new(LOCK), &me(), &removed, &mut counter(), 1763030501),
        vec![
            expected(
                "op-1",
                "takeover-guard-removal:/repo/.git/mortar/locks/land.lock.takeover:1234@mac-b:1763030400"
            ),
            expected(
                "op-2",
                "takeover-guard-removal:/repo/.git/mortar/locks/land.lock.takeover:1234@mac-b:1763030450"
            ),
        ]
    );
}

#[test]
fn removals_carried_by_a_refusal_give_their_ops() {
    let err = LockError::Held {
        path: PathBuf::from(LOCK),
        holder: LockRecord {
            pid: 1111,
            host: "mac-a".to_string(),
            key: "land:zone-c".to_string(),
            since: 1763030300,
        },
        removed_guards: vec![old_guard(1763030400), old_guard(1763030450)],
    };
    assert_eq!(
        guard_removal_op(Path::new(LOCK), &me(), err.removed_guards(), &mut counter(), 1763030501),
        vec![
            expected(
                "op-1",
                "takeover-guard-removal:/repo/.git/mortar/locks/land.lock.takeover:1234@mac-b:1763030400"
            ),
            expected(
                "op-2",
                "takeover-guard-removal:/repo/.git/mortar/locks/land.lock.takeover:1234@mac-b:1763030450"
            ),
        ]
    );
}
