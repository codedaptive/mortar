use crate::save_message::save_message;
use crate::SnapshotError;

fn reason(sentence: &str) -> String {
    match save_message(sentence) {
        Err(SnapshotError::BadSentence { reason }) => reason,
        other => panic!("expected BadSentence, got {other:?}"),
    }
}

#[test]
fn sentence_becomes_the_subject_line() {
    assert_eq!(save_message("Add the save call").unwrap(), "Add the save call\n");
}

#[test]
fn surrounding_spaces_are_kept() {
    assert_eq!(save_message("  indented subject ").unwrap(), "  indented subject \n");
}

#[test]
fn empty_sentence_is_bad() {
    assert_eq!(reason(""), "the sentence is empty or only whitespace");
}

#[test]
fn whitespace_only_sentence_is_bad() {
    assert_eq!(reason(" \t "), "the sentence is empty or only whitespace");
}

#[test]
fn newline_in_sentence_is_bad() {
    assert_eq!(reason("first line\nsecond line"), "the sentence holds a line break; a commit subject is one line");
}

#[test]
fn carriage_return_in_sentence_is_bad() {
    assert_eq!(reason("first line\rsecond"), "the sentence holds a line break; a commit subject is one line");
}
