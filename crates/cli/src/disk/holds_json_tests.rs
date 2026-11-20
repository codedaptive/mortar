//! holds_json: the path as a JSON string, the two counts, joined as true, false or null and no_joined_answer as null,
//! "a zone folder" or the reason, with no spaces.

use std::path::Path;

use folders::UnsavedCounts;

use super::{holds_json, FolderHolds, JoinedAnswer};

fn holds(tracked: u64, untracked: u64, joined: JoinedAnswer) -> FolderHolds {
    FolderHolds { unsaved: UnsavedCounts { tracked, untracked }, joined }
}

#[test]
fn a_joined_request_is_true_with_no_reason() {
    assert_eq!(
        holds_json(Path::new("/w/fix-1"), &holds(0, 0, JoinedAnswer::Joined)),
        r#"{"path":"/w/fix-1","tracked":0,"untracked":0,"joined":true,"no_joined_answer":null}"#
    );
}

#[test]
fn an_unjoined_request_is_false_with_no_reason() {
    assert_eq!(
        holds_json(Path::new("/w/fix-1"), &holds(1, 2, JoinedAnswer::NotJoined)),
        r#"{"path":"/w/fix-1","tracked":1,"untracked":2,"joined":false,"no_joined_answer":null}"#
    );
}

#[test]
fn a_zone_folder_is_null_because_it_is_a_zone_folder() {
    assert_eq!(
        holds_json(Path::new("/w/core"), &holds(12, 345, JoinedAnswer::ZoneFolder)),
        r#"{"path":"/w/core","tracked":12,"untracked":345,"joined":null,"no_joined_answer":"a zone folder"}"#
    );
}

#[test]
fn a_failed_joined_read_is_null_with_its_reason_escaped() {
    let unknown = JoinedAnswer::Unknown(r#"the size of a"b could not be read"#.to_string());
    assert_eq!(
        holds_json(Path::new("/w/fix-1"), &holds(3, 0, unknown)),
        r#"{"path":"/w/fix-1","tracked":3,"untracked":0,"joined":null,"no_joined_answer":"the size of a\"b could not be read"}"#
    );
}

#[test]
fn a_quote_and_a_backslash_in_the_path_are_escaped() {
    assert_eq!(
        holds_json(Path::new(r#"/w/a"b\c"#), &holds(0, 1, JoinedAnswer::ZoneFolder)),
        r#"{"path":"/w/a\"b\\c","tracked":0,"untracked":1,"joined":null,"no_joined_answer":"a zone folder"}"#
    );
}
