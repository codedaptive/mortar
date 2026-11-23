//! join_state_value: the name, a hand-written expected text with rebased_to and folder_tree cleared, the same text
//! for a state that had them None already, the round trip through decode_join_state, and a non-UTF-8 path refused.

use crate::decode_join_state::decode_join_state;
use crate::join_state::JoinState;
use crate::join_state_name::JOIN_STATE;
use crate::join_state_value::join_state_value;
use gitio::Oid;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn oid(digit: char) -> Oid {
    Oid::parse(&digit.to_string().repeat(40)).unwrap()
}

/// A state with every field filled: rebased_to and folder_tree Some, remaining, files and all_files non-empty.
fn rebased() -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid('1'),
        onto: oid('2'),
        replayed_tip: oid('3'),
        commit: oid('4'),
        tree: oid('5'),
        remaining: vec![oid('6'), oid('7')],
        files: vec![PathBuf::from("a.txt")],
        all_files: vec![PathBuf::from("a.txt"), PathBuf::from("dir/b.txt")],
        files_written: true,
        rebased_to: Some(oid('8')),
        folder_tree: Some(oid('9')),
        base: None,
    }
}

const PAUSED_TEXT: &str = concat!(
    "{\"request_ref\":\"refs/heads/request/r1\",",
    "\"request_tip\":\"1111111111111111111111111111111111111111\",",
    "\"onto\":\"2222222222222222222222222222222222222222\",",
    "\"replayed_tip\":\"3333333333333333333333333333333333333333\",",
    "\"commit\":\"4444444444444444444444444444444444444444\",",
    "\"tree\":\"5555555555555555555555555555555555555555\",",
    "\"remaining\":[\"6666666666666666666666666666666666666666\",\"7777777777777777777777777777777777777777\"],",
    "\"files\":[\"a.txt\"],\"all_files\":[\"a.txt\",\"dir/b.txt\"],",
    "\"files_written\":true,\"rebased_to\":null,\"folder_tree\":null}"
);

#[test]
fn the_name_is_join_state() {
    assert_eq!(JOIN_STATE, "join_state");
}

#[test]
fn a_rebased_state_is_recorded_with_rebased_to_and_folder_tree_null_and_no_newline() {
    assert_eq!(join_state_value(&rebased()), Ok(("join_state".to_string(), PAUSED_TEXT.to_string())));
}

#[test]
fn a_state_already_without_them_gives_the_same_text() {
    let paused = JoinState { rebased_to: None, folder_tree: None, ..rebased() };
    assert_eq!(join_state_value(&paused), Ok(("join_state".to_string(), PAUSED_TEXT.to_string())));
}

#[test]
fn the_value_decodes_to_the_state_with_both_cleared() {
    let states = [
        rebased(),
        JoinState { remaining: vec![], files_written: false, rebased_to: None, folder_tree: None, ..rebased() },
        JoinState { rebased_to: Some(oid('a')), folder_tree: None, ..rebased() },
        JoinState {
            files: vec![PathBuf::from("quote\"and\\slash.txt")],
            all_files: vec![PathBuf::from("quote\"and\\slash.txt"), PathBuf::from("tab\there")],
            ..rebased()
        },
    ];
    for state in states {
        let (_, text) = join_state_value(&state).unwrap();
        let expected = JoinState { rebased_to: None, folder_tree: None, ..state };
        assert_eq!(decode_join_state(&text), Ok(expected));
    }
}

#[test]
fn a_conflicted_path_that_is_not_utf8_is_the_error() {
    let bad = PathBuf::from(OsStr::from_bytes(b"bad\xff.txt"));
    let state = JoinState { files: vec![bad.clone()], all_files: vec![bad.clone()], ..rebased() };
    assert_eq!(join_state_value(&state), Err(bad));
}
