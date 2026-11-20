//! save_sentence: one non-blank, one-line argument is the sentence; no word, two words, a blank or a multi-line one
//! is refused with why.
//! Unit tests: the one-argument cases run snapshot::sentence_refusal, exported and pure on the text; the
//! report module is pure and used directly.

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
const EMPTY: &str = "the sentence is empty or only whitespace";
const BREAK: &str = "the sentence holds a line break; a commit subject is one line";

#[test]
fn no_word_is_refused() {
    assert_eq!(save_sentence(&[]), Err(refused(NOT_ONE)));
}

#[test]
fn two_words_are_refused() {
    assert_eq!(save_sentence(&words(&["fix", "parser"])), Err(refused(NOT_ONE)));
}

#[test]
fn one_sentence_is_the_word() {
    assert_eq!(save_sentence(&words(&["fix the parser"])), Ok("fix the parser"));
}

#[test]
fn an_empty_sentence_is_refused() {
    assert_eq!(save_sentence(&words(&[""])), Err(refused(EMPTY)));
}

#[test]
fn a_whitespace_sentence_is_refused() {
    assert_eq!(save_sentence(&words(&["   "])), Err(refused(EMPTY)));
}

#[test]
fn a_newline_is_refused() {
    assert_eq!(save_sentence(&words(&["a\nb"])), Err(refused(BREAK)));
}

#[test]
fn a_carriage_return_is_refused() {
    assert_eq!(save_sentence(&words(&["a\rb"])), Err(refused(BREAK)));
}
