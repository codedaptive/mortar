//! source(): the inner io::Error for Random, nothing for Malformed.

use std::error::Error;
use std::io::ErrorKind;

use super::ChangeIdError;

#[test]
fn random_source_is_the_inner_io_error() {
    let err = ChangeIdError::Random {
        source: std::io::Error::new(ErrorKind::PermissionDenied, "denied here"),
    };
    let inner = err
        .source()
        .and_then(|s| s.downcast_ref::<std::io::Error>())
        .map(std::io::Error::kind);
    assert_eq!(inner, Some(ErrorKind::PermissionDenied));
}

#[test]
fn malformed_has_no_source() {
    let err = ChangeIdError::Malformed {
        text: "abc".to_string(),
        reason: "it has 3 chars".to_string(),
    };
    assert!(err.source().is_none());
}
