use crate::changed_files_request::changed_files_request;
use crate::Oid;
use std::ffi::OsString;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_lists_names_nul_ended_without_renames() {
    let (args, _) = changed_files_request(
        &oid("4c3115d7f09202012d6f64a6b6174c6774c3192e"),
        &oid("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
    );
    let expected: Vec<OsString> = [
        "diff",
        "--name-only",
        "-z",
        "--no-renames",
        "4c3115d7f09202012d6f64a6b6174c6774c3192e",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_are_read_only_with_no_answer_codes() {
    let one = oid("4c3115d7f09202012d6f64a6b6174c6774c3192e");
    let (_, opts) = changed_files_request(&one, &one);
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
