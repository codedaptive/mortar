//! The takeover guard path beside a lock.

use std::path::{Path, PathBuf};

use super::takeover_guard_path;

#[test]
fn absolute_lock_path() {
    assert_eq!(
        takeover_guard_path(Path::new("/repo/.git/mortar/locks/land.lock")),
        PathBuf::from("/repo/.git/mortar/locks/land.lock.takeover")
    );
}

#[test]
fn relative_lock_path() {
    assert_eq!(
        takeover_guard_path(Path::new("join-zone-a.lock")),
        PathBuf::from("join-zone-a.lock.takeover")
    );
}
