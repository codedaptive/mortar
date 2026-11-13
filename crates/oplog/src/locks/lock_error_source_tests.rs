//! source(): the inner io::Error for Io, nothing for every other variant.

use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

use super::{LockError, LockRecord};

#[test]
fn io_source_is_the_inner_io_error() {
    let err = LockError::Io {
        path: PathBuf::from("/repo/.git/mortar/locks"),
        source: std::io::Error::new(ErrorKind::PermissionDenied, "denied here"),
    };
    let inner = err
        .source()
        .and_then(|s| s.downcast_ref::<std::io::Error>())
        .map(std::io::Error::kind);
    assert_eq!(inner, Some(ErrorKind::PermissionDenied));
}

#[test]
fn non_io_variants_have_no_source() {
    let held = LockError::Held {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
        holder: LockRecord {
            pid: 4242,
            host: "build-mac".to_string(),
            key: "land".to_string(),
            since: 1763030400,
        },
    };
    assert!(held.source().is_none());
    assert!(LockError::BadName("a/b".to_string()).source().is_none());
}
