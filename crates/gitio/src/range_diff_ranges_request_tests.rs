use crate::range_diff_ranges_request::range_diff_ranges_request;
use crate::Oid;
use std::ffi::OsString;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_compares_first_range_with_second_range_without_colour() {
    let (args, _) = range_diff_ranges_request(
        &oid("1111111111111111111111111111111111111111"),
        &oid("2222222222222222222222222222222222222222"),
        &oid("3333333333333333333333333333333333333333"),
        &oid("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"),
    );
    let expected: Vec<OsString> = [
        "range-diff",
        "--no-color",
        "1111111111111111111111111111111111111111..2222222222222222222222222222222222222222",
        "3333333333333333333333333333333333333333..abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_are_read_only_with_no_answer_codes() {
    let one = oid("1111111111111111111111111111111111111111");
    let two = oid("2222222222222222222222222222222222222222");
    let (_, opts) = range_diff_ranges_request(&one, &two, &two, &one);
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
