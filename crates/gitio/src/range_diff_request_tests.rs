use crate::range_diff_request::range_diff_request;
use crate::Oid;
use std::ffi::OsString;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_compares_base_to_reviewed_with_base_to_new_without_colour() {
    let (args, _) = range_diff_request(
        &oid("1111111111111111111111111111111111111111"),
        &oid("2222222222222222222222222222222222222222"),
        &oid("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"),
    );
    let expected: Vec<OsString> = [
        "range-diff",
        "--no-color",
        "1111111111111111111111111111111111111111..2222222222222222222222222222222222222222",
        "1111111111111111111111111111111111111111..abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_are_read_only_with_no_answer_codes() {
    let one = oid("1111111111111111111111111111111111111111");
    let (_, opts) = range_diff_request(&one, &one, &one);
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
