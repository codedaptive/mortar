//! encode_join_state: literal expected text written by hand, empty arrays, both flag values, a rebased tip and
//! folder tree, all_files differing from files, escapes, and a non-UTF-8 path in files and in all_files.

use crate::encode_join_state::encode_join_state;
use crate::join_state::JoinState;
use gitio::Oid;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(remaining: Vec<Oid>, files: Vec<PathBuf>) -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid("1111111111111111111111111111111111111111"),
        onto: oid("2222222222222222222222222222222222222222"),
        replayed_tip: oid("3333333333333333333333333333333333333333"),
        commit: oid("4444444444444444444444444444444444444444"),
        tree: oid("5555555555555555555555555555555555555555"),
        remaining,
        files,
        all_files: Vec::new(),
        files_written: false,
        rebased_to: None,
        folder_tree: None,
    }
}

#[test]
fn full_state_is_one_object_in_key_order_with_a_trailing_newline() {
    let s = JoinState {
        all_files: vec![PathBuf::from("old/c.md"), PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        ..state(
            vec![oid("6666666666666666666666666666666666666666"), oid("7777777777777777777777777777777777777777")],
            vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        )
    };
    assert_eq!(
        encode_join_state(&s),
        Ok("{\"request_ref\":\"refs/heads/request/r1\",\
            \"request_tip\":\"1111111111111111111111111111111111111111\",\
            \"onto\":\"2222222222222222222222222222222222222222\",\
            \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
            \"commit\":\"4444444444444444444444444444444444444444\",\
            \"tree\":\"5555555555555555555555555555555555555555\",\
            \"remaining\":[\"6666666666666666666666666666666666666666\",\
            \"7777777777777777777777777777777777777777\"],\
            \"files\":[\"src/a.rs\",\"b.txt\"],\
            \"all_files\":[\"old/c.md\",\"src/a.rs\",\"b.txt\"],\
            \"files_written\":false,\"rebased_to\":null,\"folder_tree\":null}\n"
            .to_string())
    );
}

#[test]
fn empty_remaining_files_and_all_files_are_empty_arrays() {
    assert_eq!(
        encode_join_state(&state(Vec::new(), Vec::new())),
        Ok("{\"request_ref\":\"refs/heads/request/r1\",\
            \"request_tip\":\"1111111111111111111111111111111111111111\",\
            \"onto\":\"2222222222222222222222222222222222222222\",\
            \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
            \"commit\":\"4444444444444444444444444444444444444444\",\
            \"tree\":\"5555555555555555555555555555555555555555\",\
            \"remaining\":[],\"files\":[],\"all_files\":[],\"files_written\":false,\"rebased_to\":null,\
            \"folder_tree\":null}\n"
            .to_string())
    );
}

#[test]
fn files_written_true_a_rebased_tip_and_a_folder_tree_end_the_object() {
    let s = JoinState {
        files_written: true,
        rebased_to: Some(oid("8888888888888888888888888888888888888888")),
        folder_tree: Some(oid("9999999999999999999999999999999999999999")),
        ..state(Vec::new(), Vec::new())
    };
    assert_eq!(
        encode_join_state(&s),
        Ok("{\"request_ref\":\"refs/heads/request/r1\",\
            \"request_tip\":\"1111111111111111111111111111111111111111\",\
            \"onto\":\"2222222222222222222222222222222222222222\",\
            \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
            \"commit\":\"4444444444444444444444444444444444444444\",\
            \"tree\":\"5555555555555555555555555555555555555555\",\
            \"remaining\":[],\"files\":[],\"all_files\":[],\"files_written\":true,\
            \"rebased_to\":\"8888888888888888888888888888888888888888\",\
            \"folder_tree\":\"9999999999999999999999999999999999999999\"}\n"
            .to_string())
    );
}

#[test]
fn path_with_quote_backslash_newline_and_control_chars_is_escaped() {
    let text = encode_join_state(&state(Vec::new(), vec![PathBuf::from("a\"b\\c\nd\u{1}e\tf\u{8}g\u{c}h\ri")])).unwrap();
    let escaped = "\"files\":[\"a\\\"b\\\\c\\nd\\u0001e\\tf\\bg\\fh\\ri\"],\"all_files\"";
    assert!(text.contains(escaped), "{text}");
}

#[test]
fn non_ascii_path_is_written_as_is() {
    let s =
        JoinState { all_files: vec![PathBuf::from("café.txt")], ..state(Vec::new(), vec![PathBuf::from("café.txt")]) };
    let text = encode_join_state(&s).unwrap();
    assert!(text.contains("\"files\":[\"café.txt\"],\"all_files\":[\"café.txt\"],\"files_written\""), "{text}");
}

#[test]
fn first_non_utf8_path_is_refused_naming_it() {
    let bad = PathBuf::from(OsStr::from_bytes(b"bad\xffname"));
    let worse = PathBuf::from(OsStr::from_bytes(b"other\xfe"));
    let files = vec![PathBuf::from("ok.txt"), bad, worse];
    assert_eq!(encode_join_state(&state(Vec::new(), files)), Err(PathBuf::from(OsStr::from_bytes(b"bad\xffname"))));
}

#[test]
fn a_non_utf8_path_only_in_all_files_is_refused_naming_it() {
    let s = JoinState {
        all_files: vec![PathBuf::from("ok.txt"), PathBuf::from(OsStr::from_bytes(b"earlier\xffround"))],
        ..state(Vec::new(), vec![PathBuf::from("ok.txt")])
    };
    assert_eq!(encode_join_state(&s), Err(PathBuf::from(OsStr::from_bytes(b"earlier\xffround"))));
}
