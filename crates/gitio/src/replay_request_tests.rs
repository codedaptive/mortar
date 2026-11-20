use crate::replay_request::replay_request;
use crate::Oid;
use std::ffi::OsString;

const ATTR: &str = "9a3c1e5b7d2f4a6c8e0b1d3f5a7c9e1b3d5f7a9c";
const ONTO: &str = "f67e59d9bd3426d1433ea69e2257f6dfb660acc3";
const BASE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_reads_attributes_from_attr_source_and_prints_the_moves_onto_the_new_base_for_the_range() {
    let (args, _) = replay_request(&oid(ATTR), &oid(ONTO), &oid(BASE), "refs/heads/rc").unwrap();
    let expected: Vec<OsString> = [
        "--attr-source=9a3c1e5b7d2f4a6c8e0b1d3f5a7c9e1b3d5f7a9c",
        "replay",
        "--ref-action=print",
        "--onto",
        "f67e59d9bd3426d1433ea69e2257f6dfb660acc3",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef..refs/heads/rc",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_take_exit_1_as_an_answer_and_are_not_read_only() {
    let (_, opts) = replay_request(&oid(ATTR), &oid(ONTO), &oid(BASE), "refs/heads/rc").unwrap();
    assert_eq!(opts.answer_codes, vec![1]);
    assert!(!opts.read_only);
    assert_eq!(opts.cwd, None);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn empty_tip_is_refused() {
    let err = replay_request(&oid(ATTR), &oid(ONTO), &oid(BASE), "").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable tip ref (empty or starting with '-'): \"\"");
}

#[test]
fn tip_starting_with_a_dash_is_refused() {
    let err = replay_request(&oid(ATTR), &oid(ONTO), &oid(BASE), "--contained").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not a usable tip ref (empty or starting with '-'): \"--contained\"");
}

#[test]
fn nul_in_tip_is_refused_naming_it() {
    let err = replay_request(&oid(ATTR), &oid(ONTO), &oid(BASE), "refs/heads/r\0c").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "tip ref holds a NUL: \"refs/heads/r\\0c\"");
}
