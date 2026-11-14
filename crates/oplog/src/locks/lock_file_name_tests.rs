//! Lock file names, and the zone names refused.

use super::{lock_file_name, LockError, LockName};

#[test]
fn land_is_land_lock() {
    assert_eq!(lock_file_name(&LockName::Land).unwrap(), "land.lock");
}

#[test]
fn folders_is_folders_lock() {
    assert_eq!(lock_file_name(&LockName::Folders).unwrap(), "folders.lock");
    assert_eq!(
        lock_file_name(&LockName::Join("folders".to_string())).unwrap(),
        "join-folders.lock"
    );
}

#[test]
fn join_is_join_zone_lock() {
    assert_eq!(
        lock_file_name(&LockName::Join("zone-a".to_string())).unwrap(),
        "join-zone-a.lock"
    );
    assert_eq!(
        lock_file_name(&LockName::Join("Z_9.v2".to_string())).unwrap(),
        "join-Z_9.v2.lock"
    );
}

#[test]
fn join_accepts_exactly_100_bytes() {
    let zone = "a".repeat(100);
    assert_eq!(
        lock_file_name(&LockName::Join(zone.clone())).unwrap(),
        format!("join-{zone}.lock")
    );
}

#[test]
fn join_refuses_bad_zones() {
    let long = "a".repeat(101);
    for zone in ["", "a/b", "..", ".x", "a\\b", "a\0b", "a b", long.as_str()] {
        match lock_file_name(&LockName::Join(zone.to_string())) {
            Err(LockError::BadName(got)) => assert_eq!(got, zone),
            other => panic!("zone {zone:?} gave {other:?}"),
        }
    }
}
