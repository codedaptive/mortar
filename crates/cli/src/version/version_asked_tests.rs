//! version_asked: `--version` alone or with one `--json`, in either order; anything else is not a version request.
//! ZUT (rule 11): version_asked is pure.

use super::version_asked;

fn words(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|word| word.to_string()).collect()
}

#[test]
fn version_alone_is_plain() {
    assert_eq!(version_asked(&words(&["--version"])), Some(false));
}

#[test]
fn version_with_json_in_either_order_is_json() {
    assert_eq!(version_asked(&words(&["--version", "--json"])), Some(true));
    assert_eq!(version_asked(&words(&["--json", "--version"])), Some(true));
}

#[test]
fn anything_else_is_not_a_version_request() {
    let cases: [&[&str]; 8] = [
        &[],
        &["--json"],
        &["status"],
        &["--version", "--version"],
        &["--version", "--json", "--json"],
        &["status", "--version"],
        &["--version", "--actor", "h"],
        &["-V"],
    ];
    for argv in cases {
        assert_eq!(version_asked(&words(argv)), None, "{argv:?}");
    }
}
