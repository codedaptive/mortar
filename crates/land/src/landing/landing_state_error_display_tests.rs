//! Display for LandingStateError: Corrupt as literal text; Read, Write and Remove name the path and say what
//! failed. The io error's own text is std's, so only the text around the path is asserted.

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
    assert!(text.starts_with("landing state /r/mortar/landing.json could not be read: "), "{text}");
}

#[test]
fn write_names_the_path_and_says_it_could_not_be_written() {
    let error = LandingStateError::Write { path: PathBuf::from("/r/mortar/landing.json.tmp"), source: io_error() };
    let text = error.to_string();
    assert!(text.starts_with("landing state /r/mortar/landing.json.tmp could not be written: "), "{text}");
}

#[test]
fn remove_names_the_path_and_says_it_could_not_be_removed() {
    let error = LandingStateError::Remove { path: PathBuf::from("/r/mortar/landing.json"), source: io_error() };
    let text = error.to_string();
    assert!(text.starts_with("landing state /r/mortar/landing.json could not be removed: "), "{text}");
}
