//! version_text: plain "mortar <version> <commit>" on one line; json one object {verb, ok, version, commit} on one
//! line; neither has a `next:` line.
//! ZUT: version_text is pure, and oplog's write_json_string, which it uses, is pure too.

use super::version_text;

#[test]
fn plain_is_one_line() {
    assert_eq!(version_text("0.1.0", "0123456789ab", false), "mortar 0.1.0 0123456789ab\n");
}

#[test]
fn an_unknown_commit_is_printed_as_given() {
    assert_eq!(version_text("0.1.0", "unknown", false), "mortar 0.1.0 unknown\n");
}

#[test]
fn json_is_one_object_on_one_line() {
    assert_eq!(
        version_text("0.1.0", "0123456789ab", true),
        "{\"verb\":\"version\",\"ok\":true,\"version\":\"0.1.0\",\"commit\":\"0123456789ab\"}\n"
    );
}

#[test]
fn json_strings_are_escaped() {
    assert_eq!(
        version_text("1.0.0-\"rc\"", "unknown", true),
        "{\"verb\":\"version\",\"ok\":true,\"version\":\"1.0.0-\\\"rc\\\"\",\"commit\":\"unknown\"}\n"
    );
}
