//! with_removed_guards(): a refusal that carries a list gains the given one and keeps its other fields; a variant
//! without a list comes back unchanged.

use std::path::PathBuf;

use super::{LockError, LockRecord};

fn guard(pid: u32, since: u64) -> LockRecord {
    LockRecord {
        pid,
        host: "build-mac".to_string(),
        key: "land".to_string(),
        since,
    }
}

#[test]
fn contended_gains_the_list() {
    let err = LockError::Contended {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
        removed_guards: Vec::new(),
    }
    .with_removed_guards(vec![guard(101, 1763030100), guard(202, 1763030200)]);
    match err {
        LockError::Contended { path, removed_guards } => {
            assert_eq!(path, PathBuf::from("/repo/.git/mortar/locks/land.lock"));
            assert_eq!(removed_guards, vec![guard(101, 1763030100), guard(202, 1763030200)]);
        }
        other => panic!("expected Contended: {other:?}"),
    }
}

#[test]
fn takeover_in_progress_keeps_path_and_holder_and_gains_the_list() {
    let err = LockError::TakeoverInProgress {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock.takeover"),
        holder: guard(4242, 1763030400),
        removed_guards: Vec::new(),
    }
    .with_removed_guards(vec![guard(101, 1763030100)]);
    match err {
        LockError::TakeoverInProgress {
            path,
            holder,
            removed_guards,
        } => {
            assert_eq!(path, PathBuf::from("/repo/.git/mortar/locks/land.lock.takeover"));
            assert_eq!(holder, guard(4242, 1763030400));
            assert_eq!(removed_guards, vec![guard(101, 1763030100)]);
        }
        other => panic!("expected TakeoverInProgress: {other:?}"),
    }
}

#[test]
fn bad_name_is_unchanged() {
    match LockError::BadName("a/b".to_string()).with_removed_guards(vec![guard(101, 1763030100)]) {
        LockError::BadName(zone) => assert_eq!(zone, "a/b"),
        other => panic!("expected BadName: {other:?}"),
    }
}

#[test]
fn held_keeps_path_and_holder_and_gains_the_list() {
    let err = LockError::Held {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
        holder: guard(4242, 1763030400),
        removed_guards: vec![guard(999, 1763039999)],
    }
    .with_removed_guards(vec![guard(101, 1763030100), guard(202, 1763030200)]);
    match err {
        LockError::Held {
            path,
            holder,
            removed_guards,
        } => {
            assert_eq!(path, PathBuf::from("/repo/.git/mortar/locks/land.lock"));
            assert_eq!(holder, guard(4242, 1763030400));
            assert_eq!(removed_guards, vec![guard(101, 1763030100), guard(202, 1763030200)]);
        }
        other => panic!("expected Held: {other:?}"),
    }
}

#[test]
fn malformed_keeps_path_and_reason_and_gains_the_list() {
    let err = LockError::Malformed {
        path: Some(PathBuf::from("/repo/.git/mortar/locks/land.lock.takeover")),
        reason: "has 3 lines, not 4".to_string(),
        removed_guards: vec![guard(999, 1763039999)],
    }
    .with_removed_guards(vec![guard(101, 1763030100)]);
    match err {
        LockError::Malformed {
            path,
            reason,
            removed_guards,
        } => {
            assert_eq!(path, Some(PathBuf::from("/repo/.git/mortar/locks/land.lock.takeover")));
            assert_eq!(reason, "has 3 lines, not 4");
            assert_eq!(removed_guards, vec![guard(101, 1763030100)]);
        }
        other => panic!("expected Malformed: {other:?}"),
    }
}

#[test]
fn io_keeps_path_and_source_and_gains_the_list() {
    let err = LockError::Io {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
        removed_guards: vec![guard(999, 1763039999)],
    }
    .with_removed_guards(vec![guard(101, 1763030100), guard(202, 1763030200)]);
    match err {
        LockError::Io {
            path,
            source,
            removed_guards,
        } => {
            assert_eq!(path, PathBuf::from("/repo/.git/mortar/locks/land.lock"));
            assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            assert_eq!(source.to_string(), "denied here");
            assert_eq!(removed_guards, vec![guard(101, 1763030100), guard(202, 1763030200)]);
        }
        other => panic!("expected Io: {other:?}"),
    }
}
