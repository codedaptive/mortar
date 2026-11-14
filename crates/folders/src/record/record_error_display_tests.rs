//! RecordError's text: "folder record: " and the reason; as a std error it has no source.

use std::error::Error;

use super::RecordError;

#[test]
fn text_is_folder_record_and_the_reason() {
    let err = RecordError {
        reason: "missing key \"base\"".to_string(),
    };
    assert_eq!(err.to_string(), "folder record: missing key \"base\"");
}

#[test]
fn empty_reason_keeps_the_prefix() {
    let err = RecordError {
        reason: String::new(),
    };
    assert_eq!(err.to_string(), "folder record: ");
}

#[test]
fn has_no_source() {
    let err = RecordError {
        reason: "x".to_string(),
    };
    assert!(err.source().is_none());
}
