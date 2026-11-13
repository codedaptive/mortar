//! ref_value_from_output: exit 1 is absent, exit 0 is the printed id, anything else Malformed.

use gitio::Oid;

use super::{ref_value_from_output, OplogError};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn reason(status: i32, stdout: &[u8]) -> String {
    match ref_value_from_output(status, stdout) {
        Err(OplogError::Malformed { line: None, reason }) => reason,
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn exit_one_is_absent() {
    assert_eq!(ref_value_from_output(1, b"").unwrap(), None);
}

#[test]
fn exit_zero_is_the_printed_id() {
    assert_eq!(ref_value_from_output(0, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n").unwrap(), Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")));
    assert_eq!(ref_value_from_output(0, b"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\n").unwrap(), Some(oid("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")));
    assert_eq!(ref_value_from_output(0, b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(), Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")));
}

#[test]
fn exit_zero_with_a_bad_id_names_the_text() {
    assert_eq!(
        reason(0, b"HEAD\n"),
        "git rev-parse printed \"HEAD\", which is not a git object id"
    );
    assert_eq!(
        reason(0, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\n"),
        "git rev-parse printed \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\n\", which is not a git object id"
    );
    assert_eq!(reason(0, b""), "git rev-parse printed \"\", which is not a git object id");
}

#[test]
fn other_exits_are_malformed() {
    assert_eq!(
        reason(128, b"fatal"),
        "git rev-parse exited 128 (expected 0 or 1) and printed \"fatal\""
    );
    assert_eq!(reason(2, b""), "git rev-parse exited 2 (expected 0 or 1) and printed \"\"");
}
