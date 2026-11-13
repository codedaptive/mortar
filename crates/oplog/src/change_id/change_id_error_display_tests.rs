//! Refusal texts: Malformed names the text and the reason; Random names /dev/urandom and the io error.

use std::io::ErrorKind;

use super::ChangeIdError;

#[test]
fn malformed_names_text_and_reason() {
    let err = ChangeIdError::Malformed {
        text: "abc".to_string(),
        reason: "it has 3 chars".to_string(),
    };
    assert_eq!(err.to_string(), "not a change ID: \"abc\": it has 3 chars");
}

#[test]
fn random_names_urandom_and_the_cause() {
    let err = ChangeIdError::Random {
        source: std::io::Error::new(ErrorKind::NotFound, "no such device here"),
    };
    let text = err.to_string();
    assert!(text.contains("/dev/urandom"), "{text}");
    assert!(text.contains("no such device here"), "{text}");
}
