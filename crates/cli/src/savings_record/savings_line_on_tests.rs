//! savings_line_on: off only for a top-level "savings_line": false; everything else is on.

use super::savings_line_on;

#[test]
fn off_only_for_false() {
    assert!(!savings_line_on(Some(r#"{"savings_line": false}"#)));
    assert!(!savings_line_on(Some(r#"{"other": 1, "savings_line": false}"#)));
}

#[test]
fn on_for_a_missing_file_key_or_true() {
    assert!(savings_line_on(None));
    assert!(savings_line_on(Some("{}")));
    assert!(savings_line_on(Some(r#"{"savings_line": true}"#)));
    assert!(savings_line_on(Some(r#"{"a": {"savings_line": false}}"#)));
}

#[test]
fn on_for_a_non_boolean() {
    assert!(savings_line_on(Some(r#"{"savings_line": 0}"#)));
    assert!(savings_line_on(Some(r#"{"savings_line": "false"}"#)));
    assert!(savings_line_on(Some(r#"{"savings_line": null}"#)));
}

#[test]
fn on_for_text_that_does_not_parse() {
    assert!(savings_line_on(Some(r#"{"savings_line": fals"#)));
    assert!(savings_line_on(Some("")));
    assert!(savings_line_on(Some(r#"{"savings_line": false} trailing"#)));
}
