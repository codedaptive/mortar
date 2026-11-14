use crate::first_parent_list_request::first_parent_list_request;
use crate::Oid;
use std::ffi::OsString;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_lists_base_to_tip_first_parents_oldest_first() {
    let (args, _) = first_parent_list_request(
        &oid("9145c5477bb7654747096c3188e5e006608d1c14"),
        &oid("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
    );
    let want: Vec<OsString> = [
        "rev-list",
        "--reverse",
        "--first-parent",
        "--end-of-options",
        "9145c5477bb7654747096c3188e5e006608d1c14..fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
        "--",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, want);
}

#[test]
fn argv_keeps_base_before_tip() {
    let (args, _) = first_parent_list_request(
        &oid("1111111111111111111111111111111111111111"),
        &oid("2222222222222222222222222222222222222222"),
    );
    assert_eq!(args[4], OsString::from("1111111111111111111111111111111111111111..2222222222222222222222222222222222222222"));
}

#[test]
fn opts_are_read_only_with_no_answer_codes() {
    let one = oid("9145c5477bb7654747096c3188e5e006608d1c14");
    let (_, opts) = first_parent_list_request(&one, &one);
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
