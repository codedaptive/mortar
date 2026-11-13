use crate::merge_tree_request::merge_tree_request;
use crate::Oid;
use std::ffi::OsString;

const ATTR: &str = "1111111111111111111111111111111111111111";
const BASE: &str = "2222222222222222222222222222222222222222";
const OURS: &str = "3333333333333333333333333333333333333333";
const THEIRS: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn attr_source_comes_before_the_command_and_no_base_is_left_out() {
    let (args, _) = merge_tree_request(&oid(ATTR), None, &oid(OURS), &oid(THEIRS));
    assert_eq!(
        args,
        argv(&[
            "--attr-source=1111111111111111111111111111111111111111",
            "merge-tree",
            "--write-tree",
            "-z",
            "--name-only",
            "3333333333333333333333333333333333333333",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ])
    );
}

#[test]
fn merge_base_goes_before_the_two_commits() {
    let (args, _) = merge_tree_request(&oid(ATTR), Some(&oid(BASE)), &oid(OURS), &oid(THEIRS));
    assert_eq!(
        args,
        argv(&[
            "--attr-source=1111111111111111111111111111111111111111",
            "merge-tree",
            "--write-tree",
            "-z",
            "--name-only",
            "--merge-base=2222222222222222222222222222222222222222",
            "3333333333333333333333333333333333333333",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ])
    );
}

#[test]
fn opts_take_exit_1_as_an_answer_and_are_not_read_only() {
    let (_, opts) = merge_tree_request(&oid(ATTR), None, &oid(OURS), &oid(THEIRS));
    assert_eq!(opts.answer_codes, vec![1]);
    assert!(!opts.read_only);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
