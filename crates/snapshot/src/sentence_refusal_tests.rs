use crate::sentence_refusal::sentence_refusal;

const EMPTY: &str = "the sentence is empty or only whitespace";
const LINE_BREAK: &str = "the sentence holds a line break; a commit subject is one line";

#[test]
fn empty_sentence_is_refused_as_empty() {
    assert_eq!(sentence_refusal(""), Some(EMPTY));
}

#[test]
fn whitespace_only_sentence_is_refused_as_empty() {
    assert_eq!(sentence_refusal(" \t "), Some(EMPTY));
}

#[test]
fn newline_inside_is_refused_as_line_break() {
    assert_eq!(sentence_refusal("a\nb"), Some(LINE_BREAK));
}

#[test]
fn carriage_return_inside_is_refused_as_line_break() {
    assert_eq!(sentence_refusal("a\rb"), Some(LINE_BREAK));
}

#[test]
fn crlf_inside_is_refused_as_line_break() {
    assert_eq!(sentence_refusal("a\r\nb"), Some(LINE_BREAK));
}

#[test]
fn lone_newline_is_refused_as_empty_because_empty_is_checked_first() {
    assert_eq!(sentence_refusal("\n"), Some(EMPTY));
}

#[test]
fn plain_sentence_is_accepted() {
    assert_eq!(sentence_refusal("Add the save call"), None);
}

#[test]
fn surrounding_spaces_are_accepted() {
    assert_eq!(sentence_refusal("  spaced  "), None);
}

#[test]
fn tab_inside_is_accepted() {
    assert_eq!(sentence_refusal("a\tb"), None);
}
