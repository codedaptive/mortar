//! commit_subject: the first paragraph folded into one line, like git's %s but with each line trimmed at both ends.

use super::commit_subject;

#[test]
fn a_one_line_message_is_its_subject() {
    assert_eq!(commit_subject("Add squash ref\n"), "Add squash ref");
}

#[test]
fn the_body_after_the_blank_line_is_left_out() {
    assert_eq!(commit_subject("Add squash ref\n\nThe body says why.\nMortar-Request: r1\n"), "Add squash ref");
}

#[test]
fn a_multi_line_first_paragraph_is_folded_with_one_space() {
    assert_eq!(
        commit_subject("Add the squash ref\n  and its op  \nfor land\n\nbody\n"),
        "Add the squash ref and its op for land"
    );
}

#[test]
fn leading_blank_lines_are_skipped() {
    assert_eq!(commit_subject("\n  \nSubject here\n\nbody"), "Subject here");
}

#[test]
fn a_whitespace_only_line_ends_the_paragraph() {
    assert_eq!(commit_subject("first\n \t \nsecond\n"), "first");
}

#[test]
fn crlf_line_ends_are_trimmed() {
    assert_eq!(commit_subject("one\r\ntwo\r\n\r\nbody\r\n"), "one two");
}

#[test]
fn an_empty_message_has_an_empty_subject() {
    assert_eq!(commit_subject(""), "");
}
