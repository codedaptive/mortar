use crate::maintenance_request::maintenance_request;
use std::ffi::OsString;

#[test]
fn argv_runs_the_gc_task() {
    let (args, _) = maintenance_request();
    let expected: Vec<OsString> = ["maintenance", "run", "--task=gc"].iter().map(OsString::from).collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_write_in_the_repo() {
    let (_, opts) = maintenance_request();
    assert!(!opts.read_only);
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}
