//! answer_chars: the characters of the plain body, without the savings line, counted as chars, not bytes.

use super::{answer_chars, Savings};
use crate::report::{ExitCode, OutValue, Report};

fn report(lines: &[&str], fields: Vec<(String, OutValue)>) -> Report {
    Report {
        verb: "status".to_string(),
        exit: ExitCode::Done,
        outcome: "clean".to_string(),
        lines: lines.iter().map(|l| l.to_string()).collect(),
        owner: None,
        next: "x".to_string(),
        fields,
    }
}

fn saved() -> (String, OutValue) {
    ("savings".to_string(), OutValue::Saved(Savings { baseline: "b".to_string(), baseline_chars: 4000 }))
}

#[test]
fn counts_the_body_only() {
    // "ab\n" + "next: x\n" = 3 + 8 characters.
    assert_eq!(answer_chars(&report(&["ab"], vec![saved()])), 11);
}

#[test]
fn counts_characters_not_bytes() {
    // "é≈\n" is 3 characters (6 bytes), then "next: x\n" 8.
    assert_eq!(answer_chars(&report(&["é≈"], vec![saved()])), 11);
}

#[test]
fn the_same_with_or_without_savings() {
    let text = ("k".to_string(), OutValue::Text("v".to_string()));
    assert_eq!(answer_chars(&report(&["a", "b"], vec![text.clone(), saved()])), 12);
    assert_eq!(answer_chars(&report(&["a", "b"], vec![text])), 12);
}
