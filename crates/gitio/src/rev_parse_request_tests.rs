use crate::rev_parse_request::rev_parse_request;
use std::ffi::OsString;

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn argv_peels_to_a_commit_after_end_of_options() {
    let (args, _) = rev_parse_request("main").unwrap();
    assert_eq!(args, argv(&["rev-parse", "--verify", "-q", "--end-of-options", "main^{commit}"]));
}

#[test]
fn rev_starting_with_a_dash_stays_after_end_of_options() {
    let (args, _) = rev_parse_request("--all").unwrap();
    assert_eq!(args, argv(&["rev-parse", "--verify", "-q", "--end-of-options", "--all^{commit}"]));
}

#[test]
fn opts_are_read_only_with_exit_1_an_answer() {
    let (_, opts) = rev_parse_request("HEAD~2").unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![1]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn empty_rev_is_refused() {
    let err = rev_parse_request("").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn nul_in_rev_is_refused_naming_it() {
    let err = rev_parse_request("main\0--all").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"main\\0--all\""), "{}", err.stderr);
}
