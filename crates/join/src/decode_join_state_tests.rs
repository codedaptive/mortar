//! decode_join_state: a hand-written text read into the expected state; JSON escapes encode_join_state never writes
//! read as JSON; an unreadable string refused saying why; and refusal of keys out of order, a missing key, a bad id
//! or flag, trailing junk, spaces, bad arrays, empty text.

use crate::decode_join_state::decode_join_state;
use crate::join_state::JoinState;
use gitio::Oid;
use std::path::PathBuf;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// A valid text with the given remaining, files, files_written and rebased_to values, as raw JSON.
fn text(remaining: &str, files: &str, files_written: &str, rebased_to: &str) -> String {
    format!(
        "{{\"request_ref\":\"refs/heads/request/r1\",\
         \"request_tip\":\"1111111111111111111111111111111111111111\",\
         \"onto\":\"2222222222222222222222222222222222222222\",\
         \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
         \"commit\":\"4444444444444444444444444444444444444444\",\
         \"tree\":\"5555555555555555555555555555555555555555\",\
         \"remaining\":{remaining},\"files\":{files},\"files_written\":{files_written},\"rebased_to\":{rebased_to}}}\n"
    )
}

fn base() -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid("1111111111111111111111111111111111111111"),
        onto: oid("2222222222222222222222222222222222222222"),
        replayed_tip: oid("3333333333333333333333333333333333333333"),
        commit: oid("4444444444444444444444444444444444444444"),
        tree: oid("5555555555555555555555555555555555555555"),
        remaining: Vec::new(),
        files: Vec::new(),
        files_written: false,
        rebased_to: None,
    }
}

#[test]
fn hand_written_text_reads_into_every_field() {
    let input = text(
        "[\"6666666666666666666666666666666666666666\",\"7777777777777777777777777777777777777777\"]",
        "[\"src/a.rs\",\"say \\\"hi\\\"\\n.txt\"]",
        "true",
        "\"8888888888888888888888888888888888888888\"",
    );
    assert_eq!(
        decode_join_state(&input),
        Ok(JoinState {
            remaining: vec![
                oid("6666666666666666666666666666666666666666"),
                oid("7777777777777777777777777777777777777777"),
            ],
            files: vec![PathBuf::from("src/a.rs"), PathBuf::from("say \"hi\"\n.txt")],
            files_written: true,
            rebased_to: Some(oid("8888888888888888888888888888888888888888")),
            ..base()
        })
    );
}

#[test]
fn empty_arrays_false_and_null_read_back() {
    assert_eq!(decode_join_state(&text("[]", "[]", "false", "null")), Ok(base()));
}

#[test]
fn the_trailing_newline_is_optional() {
    let input = text("[]", "[]", "false", "null");
    assert_eq!(decode_join_state(input.trim_end_matches('\n')), Ok(base()));
}

#[test]
fn keys_out_of_order_are_refused() {
    let swapped = text("[]", "[]", "false", "null").replace(
        "\"onto\":\"2222222222222222222222222222222222222222\",\
         \"replayed_tip\":\"3333333333333333333333333333333333333333\"",
        "\"replayed_tip\":\"3333333333333333333333333333333333333333\",\
         \"onto\":\"2222222222222222222222222222222222222222\"",
    );
    assert!(swapped.find("\"replayed_tip\"") < swapped.find("\"onto\""), "{swapped}");
    assert!(decode_join_state(&swapped).is_err());
}

#[test]
fn a_missing_key_is_refused() {
    let tree_key = ",\"tree\":\"5555555555555555555555555555555555555555\"";
    let without_tree = text("[]", "[]", "false", "null").replace(tree_key, "");
    assert!(!without_tree.contains("tree\""));
    assert!(decode_join_state(&without_tree).is_err());
    let without_rebased = "{\"request_ref\":\"r\",\"request_tip\":\"1111111111111111111111111111111111111111\",\
        \"onto\":\"2222222222222222222222222222222222222222\",\
        \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
        \"commit\":\"4444444444444444444444444444444444444444\",\
        \"tree\":\"5555555555555555555555555555555555555555\",\
        \"remaining\":[],\"files\":[],\"files_written\":false}\n";
    assert!(decode_join_state(without_rebased).is_err());
}

#[test]
fn a_bad_object_id_is_refused() {
    let short = text("[]", "[]", "false", "null").replace("1111111111111111111111111111111111111111", "1111");
    assert!(decode_join_state(&short).is_err());
    let upper = text("[\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\"]", "[]", "false", "null");
    assert!(decode_join_state(&upper).is_err());
    assert!(decode_join_state(&text("[]", "[]", "false", "\"zz\"")).is_err());
}

#[test]
fn trailing_junk_is_refused() {
    let input = text("[]", "[]", "false", "null");
    for junk in [" ", "\n", "x", "{}"] {
        assert!(decode_join_state(&format!("{input}{junk}")).is_err(), "{junk:?}");
    }
    let body = input.trim_end_matches('\n');
    assert!(decode_join_state(&format!("{body},\"extra\":1}}\n")).is_err());
}

#[test]
fn a_boolean_that_is_not_true_or_false_is_refused() {
    for value in ["\"true\"", "1", "null", "True", "tru", ""] {
        assert!(decode_join_state(&text("[]", "[]", value, "null")).is_err(), "{value}");
    }
}

#[test]
fn whitespace_and_malformed_arrays_are_refused() {
    let spaced = text("[]", "[]", "false", "null").replace("\"onto\":", "\"onto\": ");
    assert!(decode_join_state(&spaced).is_err());
    for files in ["[\"a\",]", "[,\"a\"]", "[\"a\" ]", "[\"a\"", "\"a\"", "[\"a\"\"b\"]"] {
        assert!(decode_join_state(&text("[]", files, "false", "null")).is_err(), "{files}");
    }
    assert!(decode_join_state("").is_err());
}

#[test]
fn escapes_encode_join_state_never_writes_are_read_as_json() {
    let files = "[\"a\\/b\",\"\\u0041\\u00E9\",\"\\ud83d\\ude00\"]";
    assert_eq!(
        decode_join_state(&text("[]", files, "false", "null")),
        Ok(JoinState { files: vec![PathBuf::from("a/b"), PathBuf::from("Aé"), PathBuf::from("😀")], ..base() })
    );
}

#[test]
fn a_string_that_cannot_be_read_is_refused_saying_why() {
    let unknown = decode_join_state(&text("[]", "[\"a\\qb\"]", "false", "null")).unwrap_err();
    assert!(unknown.starts_with("the state file's text could not be read: "), "{unknown}");
    assert!(unknown.contains("\\q"), "{unknown}");
    let ended = decode_join_state("{\"request_ref\":\"refs/heads/requ").unwrap_err();
    assert_eq!(
        ended,
        "the state file's text could not be read: it ends inside the string starting at byte 15"
    );
}
