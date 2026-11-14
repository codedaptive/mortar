//! source(): the io error for Read, Write and Remove; none for Corrupt.

use super::LandingStateError;
use std::error::Error;
use std::io;
use std::path::PathBuf;

fn path() -> PathBuf {
    PathBuf::from("/r/mortar/landing.json")
}

/// The kind of the io error the source holds, or None when there is no io source.
fn source_kind(error: &LandingStateError) -> Option<io::ErrorKind> {
    error.source().and_then(|s| s.downcast_ref::<io::Error>()).map(io::Error::kind)
}

#[test]
fn read_source_is_the_io_error() {
    let error = LandingStateError::Read { path: path(), source: io::Error::from(io::ErrorKind::PermissionDenied) };
    assert_eq!(source_kind(&error), Some(io::ErrorKind::PermissionDenied));
}

#[test]
fn write_source_is_the_io_error() {
    let error = LandingStateError::Write { path: path(), source: io::Error::from(io::ErrorKind::WriteZero) };
    assert_eq!(source_kind(&error), Some(io::ErrorKind::WriteZero));
}

#[test]
fn remove_source_is_the_io_error() {
    let error = LandingStateError::Remove { path: path(), source: io::Error::from(io::ErrorKind::Interrupted) };
    assert_eq!(source_kind(&error), Some(io::ErrorKind::Interrupted));
}

#[test]
fn corrupt_has_no_source() {
    let error = LandingStateError::Corrupt { path: path(), reason: "bad".to_string() };
    assert!(error.source().is_none());
}
