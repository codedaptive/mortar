//! holds_json: the path as a JSON string, the two counts and joined as true, false or null, with no spaces.

use std::path::Path;

use folders::UnsavedCounts;

use super::{holds_json, FolderHolds};

fn holds(tracked: u64, untracked: u64, joined: Option<bool>) -> FolderHolds {
    FolderHolds { unsaved: UnsavedCounts { tracked, untracked }, joined }
}

#[test]
fn a_joined_request_is_true() {
    assert_eq!(
        holds_json(Path::new("/w/fix-1"), &holds(0, 0, Some(true))),
        r#"{"path":"/w/fix-1","tracked":0,"untracked":0,"joined":true}"#
    );
}

#[test]
fn an_unjoined_request_is_false() {
    assert_eq!(
        holds_json(Path::new("/w/fix-1"), &holds(1, 2, Some(false))),
        r#"{"path":"/w/fix-1","tracked":1,"untracked":2,"joined":false}"#
    );
}

#[test]
fn a_zone_folder_is_null() {
    assert_eq!(
        holds_json(Path::new("/w/core"), &holds(12, 345, None)),
        r#"{"path":"/w/core","tracked":12,"untracked":345,"joined":null}"#
    );
}

#[test]
fn a_quote_and_a_backslash_in_the_path_are_escaped() {
    assert_eq!(
        holds_json(Path::new(r#"/w/a"b\c"#), &holds(0, 1, None)),
        r#"{"path":"/w/a\"b\\c","tracked":0,"untracked":1,"joined":null}"#
    );
}
