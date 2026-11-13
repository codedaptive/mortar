use crate::blob_bytes_request::blob_bytes_request;
use crate::Oid;
use std::ffi::OsString;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_prints_the_blob() {
    let (args, _) = blob_bytes_request(&oid("5626abf0f72e58d7a153368ba57db4c673c0e171"));
    let expected: Vec<OsString> =
        ["cat-file", "blob", "5626abf0f72e58d7a153368ba57db4c673c0e171"].iter().map(OsString::from).collect();
    assert_eq!(args, expected);
}

#[test]
fn argv_takes_a_sha256_id() {
    let (args, _) = blob_bytes_request(&oid("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
    assert_eq!(args[2], OsString::from("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
}

#[test]
fn opts_are_read_only_in_the_repo() {
    let (_, opts) = blob_bytes_request(&oid("5626abf0f72e58d7a153368ba57db4c673c0e171"));
    assert!(opts.read_only);
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}
