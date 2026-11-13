//! Lock paths under the git common directory.

use std::path::{Path, PathBuf};

use super::{lock_path, LockError, LockName};

#[test]
fn land_path() {
    assert_eq!(
        lock_path(Path::new("/repo/.git"), &LockName::Land).unwrap(),
        PathBuf::from("/repo/.git/mortar/locks/land.lock")
    );
}

#[test]
fn join_path() {
    assert_eq!(
        lock_path(Path::new("/repo/.git"), &LockName::Join("zone-a".to_string())).unwrap(),
        PathBuf::from("/repo/.git/mortar/locks/join-zone-a.lock")
    );
}

#[test]
fn bad_zone_is_refused() {
    assert!(matches!(
        lock_path(Path::new("/repo/.git"), &LockName::Join("../x".to_string())),
        Err(LockError::BadName(z)) if z == "../x"
    ));
}
