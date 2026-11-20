//! Display for LandingStateError: Corrupt, Read, Write and Remove asserted as full literal text; the io error is a
//! fixed io::Error::other value.

use super::LandingStateError;
use std::io;
use std::path::PathBuf;

fn io_error() -> io::Error {
    io::Error::other("boom")
}

#[test]
fn corrupt_names_the_path_and_the_reason() {
    let error = LandingStateError::Corrupt {
        path: PathBuf::from("/repo/.git/mortar/landing.json"),
        reason: "expected {\"change_id\": at byte 0".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "landing state /repo/.git/mortar/landing.json is not a state land wrote: expected {\"change_id\": at byte 0"
    );
}

#[test]
fn read_names_the_path_and_says_it_could_not_be_read() {
    let error = LandingStateError::Read { path: PathBuf::from("/r/mortar/landing.json"), source: io_error() };
    let text = error.to_string();
    assert_eq!(text, "landing state /r/mortar/landing.json could not be read: boom");
}

#[test]
fn write_names_the_path_and_says_it_could_not_be_written() {
    let error = LandingStateError::Write { path: PathBuf::from("/r/mortar/landing.json.tmp"), source: io_error() };
    let text = error.to_string();
    assert_eq!(text, "landing state /r/mortar/landing.json.tmp could not be written: boom");
}

#[test]
fn remove_names_the_path_and_says_it_could_not_be_removed() {
    let error = LandingStateError::Remove { path: PathBuf::from("/r/mortar/landing.json"), source: io_error() };
    let text = error.to_string();
    assert_eq!(text, "landing state /r/mortar/landing.json could not be removed: boom");
}
