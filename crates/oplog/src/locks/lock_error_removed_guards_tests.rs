//! removed_guards(): the list a refusal carries, in order, and nothing for a variant without one.

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
fn held_gives_both_guards_in_order() {
    let err = LockError::Held {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
        holder: guard(4242, 1763030400),
        removed_guards: vec![guard(101, 1763030100), guard(202, 1763030200)],
    };
    assert_eq!(err.removed_guards(), &[guard(101, 1763030100), guard(202, 1763030200)][..]);
}

#[test]
fn io_gives_its_one_guard() {
    let err = LockError::Io {
        path: PathBuf::from("/repo/.git/mortar/locks"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
        removed_guards: vec![guard(303, 1763030300)],
    };
    assert_eq!(err.removed_guards(), &[guard(303, 1763030300)][..]);
}

#[test]
fn bad_name_gives_none() {
    assert!(LockError::BadName("a/b".to_string()).removed_guards().is_empty());
}
