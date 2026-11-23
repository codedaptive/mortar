//! decode_join_state: a hand-written text read into the expected state, all_files differing from files and a
//! folder tree included; JSON escapes encode_join_state never writes read as JSON; an unreadable string refused saying
//! why; and refusal of keys out of order, a missing key (all_files and folder_tree included), a bad id or flag,
//! trailing junk, spaces, bad arrays, empty text. The base keys: a text without them (as a plain join writes it)
//! reads with no base; a base with two partners and with none read back; a truncated or misordered base key refused
//! naming its byte; and encode_join_state's output read back to the state it encoded.

use crate::decode_join_state::decode_join_state;
use crate::encode_join_state::encode_join_state;
use crate::join_state::JoinState;
use crate::partner::Partner;
use crate::request_base::RequestBase;
use gitio::Oid;
use std::path::PathBuf;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// A valid text with the given remaining, files, all_files, files_written, rebased_to and folder_tree values, as raw
/// JSON.
fn full_text(
    remaining: &str,
    files: &str,
    all_files: &str,
    files_written: &str,
    rebased_to: &str,
    folder_tree: &str,
) -> String {
    format!(
        "{{\"request_ref\":\"refs/heads/request/r1\",\
         \"request_tip\":\"1111111111111111111111111111111111111111\",\
         \"onto\":\"2222222222222222222222222222222222222222\",\
         \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
         \"commit\":\"4444444444444444444444444444444444444444\",\
         \"tree\":\"5555555555555555555555555555555555555555\",\
         \"remaining\":{remaining},\"files\":{files},\"all_files\":{all_files},\"files_written\":{files_written},\
         \"rebased_to\":{rebased_to},\"folder_tree\":{folder_tree}}}\n"
    )
}

/// A valid text with an empty all_files and a null folder_tree.
fn text(remaining: &str, files: &str, files_written: &str, rebased_to: &str) -> String {
    full_text(remaining, files, "[]", files_written, rebased_to, "null")
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
        all_files: Vec::new(),
        files_written: false,
        rebased_to: None,
        folder_tree: None,
        base: None,
    }
}

#[test]
fn hand_written_text_reads_into_every_field() {
    let input = full_text(
        "[\"6666666666666666666666666666666666666666\",\"7777777777777777777777777777777777777777\"]",
        "[\"src/a.rs\",\"say \\\"hi\\\"\\n.txt\"]",
        "[\"old/c.md\",\"src/a.rs\",\"say \\\"hi\\\"\\n.txt\"]",
        "true",
        "\"8888888888888888888888888888888888888888\"",
        "\"9999999999999999999999999999999999999999\"",
    );
    assert_eq!(
        decode_join_state(&input),
        Ok(JoinState {
            remaining: vec![
                oid("6666666666666666666666666666666666666666"),
                oid("7777777777777777777777777777777777777777"),
            ],
            files: vec![PathBuf::from("src/a.rs"), PathBuf::from("say \"hi\"\n.txt")],
            all_files: vec![PathBuf::from("old/c.md"), PathBuf::from("src/a.rs"), PathBuf::from("say \"hi\"\n.txt")],
            files_written: true,
            rebased_to: Some(oid("8888888888888888888888888888888888888888")),
            folder_tree: Some(oid("9999999999999999999999999999999999999999")),
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
    assert!(!without_tree.contains("\"tree\""));
    assert!(decode_join_state(&without_tree).is_err());
    let without_rebased = "{\"request_ref\":\"r\",\"request_tip\":\"1111111111111111111111111111111111111111\",\
        \"onto\":\"2222222222222222222222222222222222222222\",\
        \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
        \"commit\":\"4444444444444444444444444444444444444444\",\
        \"tree\":\"5555555555555555555555555555555555555555\",\
        \"remaining\":[],\"files\":[],\"all_files\":[],\"files_written\":false,\"folder_tree\":null}\n";
    assert!(decode_join_state(without_rebased).is_err());
}

#[test]
fn a_text_missing_all_files_is_refused() {
    let without_all_files = "{\"request_ref\":\"r\",\"request_tip\":\"1111111111111111111111111111111111111111\",\
        \"onto\":\"2222222222222222222222222222222222222222\",\
        \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
        \"commit\":\"4444444444444444444444444444444444444444\",\
        \"tree\":\"5555555555555555555555555555555555555555\",\
        \"remaining\":[],\"files\":[\"a.txt\"],\"files_written\":false,\"rebased_to\":null,\"folder_tree\":null}\n";
    assert!(decode_join_state(without_all_files).is_err());
}

#[test]
fn a_text_missing_folder_tree_is_refused() {
    let without_folder_tree = "{\"request_ref\":\"r\",\"request_tip\":\"1111111111111111111111111111111111111111\",\
        \"onto\":\"2222222222222222222222222222222222222222\",\
        \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
        \"commit\":\"4444444444444444444444444444444444444444\",\
        \"tree\":\"5555555555555555555555555555555555555555\",\
        \"remaining\":[],\"files\":[\"a.txt\"],\"all_files\":[\"a.txt\"],\"files_written\":true,\
        \"rebased_to\":\"8888888888888888888888888888888888888888\"}\n";
    assert!(decode_join_state(without_folder_tree).is_err());
}

#[test]
fn a_folder_tree_that_is_not_an_object_id_is_refused() {
    assert!(decode_join_state(&full_text("[]", "[]", "[]", "false", "null", "\"zz\"")).is_err());
    assert!(decode_join_state(&full_text("[]", "[]", "[]", "false", "null", "\"\"")).is_err());
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

/// A valid 12-key text with `tail` written between folder_tree's value and the closing brace.
fn with_tail(tail: &str) -> String {
    let old = text("[]", "[]", "false", "null");
    format!("{}{tail}}}\n", old.strip_suffix("}\n").unwrap())
}

fn partner(zone: &str, tip: &str) -> Partner {
    Partner { zone: zone.to_string(), tip: oid(tip) }
}

#[test]
fn the_base_keys_read_into_the_base_and_their_absence_into_none() {
    let old = "{\"request_ref\":\"refs/heads/request/r1\",\
        \"request_tip\":\"1111111111111111111111111111111111111111\",\
        \"onto\":\"2222222222222222222222222222222222222222\",\
        \"replayed_tip\":\"3333333333333333333333333333333333333333\",\
        \"commit\":\"4444444444444444444444444444444444444444\",\
        \"tree\":\"5555555555555555555555555555555555555555\",\
        \"remaining\":[],\"files\":[],\"all_files\":[],\"files_written\":false,\"rebased_to\":null,\
        \"folder_tree\":null}\n";
    let two = with_tail(
        ",\"base\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"partners\":[\
         {\"zone\":\"docs\",\"tip\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"},\
         {\"zone\":\"we\\\"ird\",\"tip\":\"cccccccccccccccccccccccccccccccccccccccc\"}]",
    );
    let none = with_tail(",\"base\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"partners\":[]");
    let rows = [
        (old.to_string(), None),
        (
            two,
            Some(RequestBase {
                base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                partners: vec![
                    partner("docs", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                    partner("we\"ird", "cccccccccccccccccccccccccccccccccccccccc"),
                ],
            }),
        ),
        (none, Some(RequestBase { base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), partners: Vec::new() })),
    ];
    for (input, expected) in rows {
        assert_eq!(decode_join_state(&input), Ok(JoinState { base: expected, ..base() }), "{input}");
    }
}

#[test]
fn a_truncated_or_misordered_base_key_is_refused_naming_its_byte() {
    // The byte where folder_tree's value ends and the base keys would start.
    let start = text("[]", "[]", "false", "null").len() - 2;
    let base_hex = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";
    let rows = [
        (",\"bas".to_string(), start),
        (format!(",\"partners\":[],\"base\":{base_hex}"), start),
        (format!(",\"base\":{base_hex},\"partners\":[{{\"tip\":{base_hex},\"zone\":\"docs\"}}]"), start + 63),
        (format!(",\"base\":{base_hex},\"partners\":[{{\"zone\":\"docs\"}}]"), start + 77),
        (format!(",\"base\":{base_hex},\"partners\":[],\"extra\":1"), start + 64),
    ];
    for (tail, byte) in rows {
        let refused = decode_join_state(&with_tail(&tail)).unwrap_err();
        assert!(refused.ends_with(&format!(" at byte {byte}")), "{tail}: {refused}");
    }
}

#[test]
fn encode_join_state_output_reads_back_to_the_state() {
    let bases = [
        None,
        Some(RequestBase { base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), partners: Vec::new() }),
        Some(RequestBase {
            base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            partners: vec![
                partner("docs", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                partner("z\u{1}\\", "cccccccccccccccccccccccccccccccccccccccc"),
            ],
        }),
    ];
    for request_base in bases {
        let state = JoinState {
            remaining: vec![oid("6666666666666666666666666666666666666666")],
            files: vec![PathBuf::from("src/a.rs")],
            all_files: vec![PathBuf::from("old.md"), PathBuf::from("src/a.rs")],
            files_written: true,
            rebased_to: Some(oid("8888888888888888888888888888888888888888")),
            folder_tree: Some(oid("9999999999999999999999999999999999999999")),
            base: request_base,
            ..base()
        };
        assert_eq!(decode_join_state(&encode_join_state(&state).unwrap()), Ok(state.clone()));
    }
}
