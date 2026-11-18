//! save_sentence: no word and two words are refused with why. The one-argument cases run snapshot::sentence_refusal
//! for real, so they stay in crates/cli/tests/save_sentence_tests.rs, held for Nagatha (S-101).
//! ZUT (rule 11): save_error_report and the report module are pure and used directly.

use super::save_sentence;
use crate::report::{ExitCode, Report};

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| word.to_string()).collect()
}

fn refused(why: &str) -> Report {
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![why.to_string(), "nothing ran".to_string()],
        owner: None,
        next: "mr save \"<one line saying what changed>\"".to_string(),
        fields: Vec::new(),
    }
}

const NOT_ONE: &str = "save takes one sentence, quoted as one argument";

#[test]
fn no_word_is_refused() {
    assert_eq!(save_sentence(&[]), Err(refused(NOT_ONE)));
}

#[test]
fn two_words_are_refused() {
    assert_eq!(save_sentence(&words(&["fix", "parser"])), Err(refused(NOT_ONE)));
}
