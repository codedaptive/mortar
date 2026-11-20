use crate::remote_push_urls_request::remote_push_urls_request;
use std::ffi::OsString;

#[test]
fn argv_puts_the_remote_after_end_of_options() {
    let (args, _) = remote_push_urls_request("origin").unwrap();
    let want: Vec<OsString> = ["remote", "get-url", "--push", "--all", "--end-of-options", "origin"]
        .iter()
        .map(OsString::from)
        .collect();
    assert_eq!(args, want);
}

#[test]
fn opts_are_read_only_with_only_exit_0_an_answer() {
    let (_, opts) = remote_push_urls_request("origin").unwrap();
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn leading_dash_is_refused_before_git_runs() {
    let err = remote_push_urls_request("-oops").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable remote: \"-oops\"");
}

#[test]
fn empty_remote_is_refused() {
    let err = remote_push_urls_request("").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable remote: \"\"");
}
