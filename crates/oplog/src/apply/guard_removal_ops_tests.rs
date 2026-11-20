//! guard_removal_ops: one takeover-guard-removal op per removed guard, from a won lock or a refusal; none otherwise.

use std::path::{Path, PathBuf};

use crate::locks::{AcquiredLock, LockError, LockRecord};
use crate::store::Op;

use super::guard_removal_ops::guard_removal_ops;

fn record(pid: u32, host: &str, key: &str, since: u64) -> LockRecord {
    LockRecord {
        pid,
        host: host.to_string(),
        key: key.to_string(),
        since,
    }
}

fn won(removed_guards: Vec<LockRecord>) -> Result<AcquiredLock, LockError> {
    Ok(AcquiredLock {
        path: PathBuf::from("/won/mortar/locks/folders.lock"),
        record: record(100, "alpha", "make-zone", 1_700_000_100),
        took_over: None,
        removed_guards,
    })
}

/// Ops for `acquired`, passing a lock path and `me` that differ from a won lock's, with ids "op-1", "op-2", ...
fn ops_for(acquired: &Result<AcquiredLock, LockError>) -> Vec<Op> {
    let mut n = 0;
    let mut next = || {
        n += 1;
        format!("op-{n}")
    };
    guard_removal_ops(
        acquired,
        Path::new("/asked/mortar/locks/folders.lock"),
        &record(200, "beta", "make-request", 1_700_000_200),
        &mut next,
        1_700_000_300,
    )
}

#[test]
fn a_won_lock_logs_each_removed_guard_in_order_as_its_holder() {
    let ops = ops_for(&won(vec![
        record(11, "alpha", "old-one", 1_600_000_001),
        record(22, "alpha", "old-two", 1_600_000_002),
    ]));
    let summary: Vec<_> = ops
        .iter()
        .map(|op| (op.op_id.as_str(), op.kind.as_str(), op.key.as_str(), op.actor.as_str(), op.at, op.moves.len()))
        .collect();
    assert_eq!(
        summary,
        vec![
            (
                "op-1",
                "takeover-guard-removal",
                "takeover-guard-removal:/won/mortar/locks/folders.lock.takeover:11@alpha:1600000001",
                "100@alpha",
                1_700_000_300,
                0,
            ),
            (
                "op-2",
                "takeover-guard-removal",
                "takeover-guard-removal:/won/mortar/locks/folders.lock.takeover:22@alpha:1600000002",
                "100@alpha",
                1_700_000_300,
                0,
            ),
        ]
    );
}

#[test]
fn a_contended_refusal_logs_its_guard_under_the_asked_lock_and_me() {
    let ops = ops_for(&Err(LockError::Contended {
        path: PathBuf::from("/elsewhere/folders.lock"),
        removed_guards: vec![record(33, "beta", "old-three", 1_600_000_003)],
    }));
    let summary: Vec<_> = ops.iter().map(|op| (op.op_id.as_str(), op.key.as_str(), op.actor.as_str())).collect();
    assert_eq!(
        summary,
        vec![(
            "op-1",
            "takeover-guard-removal:/asked/mortar/locks/folders.lock.takeover:33@beta:1600000003",
            "200@beta",
        )]
    );
}

#[test]
fn a_held_refusal_logs_its_guard() {
    let ops = ops_for(&Err(LockError::Held {
        path: PathBuf::from("/asked/mortar/locks/folders.lock"),
        holder: record(44, "gamma", "live-holder", 1_600_000_004),
        removed_guards: vec![record(55, "beta", "old-five", 1_600_000_005)],
    }));
    let keys: Vec<_> = ops.iter().map(|op| op.key.as_str()).collect();
    assert_eq!(keys, vec!["takeover-guard-removal:/asked/mortar/locks/folders.lock.takeover:55@beta:1600000005"]);
}

#[test]
fn a_won_lock_with_no_removed_guard_logs_nothing() {
    assert_eq!(ops_for(&won(vec![])), vec![]);
}

#[test]
fn a_contended_refusal_with_no_removed_guard_logs_nothing() {
    let ops = ops_for(&Err(LockError::Contended {
        path: PathBuf::from("/asked/mortar/locks/folders.lock"),
        removed_guards: vec![],
    }));
    assert_eq!(ops, vec![]);
}

#[test]
fn a_bad_name_refusal_logs_nothing() {
    assert_eq!(ops_for(&Err(LockError::BadName("no/slash".to_string()))), vec![]);
}
