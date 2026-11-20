use crate::check_ref_format_request::check_ref_format_request;
use std::ffi::OsString;

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn argv_is_the_command_and_the_name() {
    let (args, _) = check_ref_format_request("refs/heads/main").unwrap();
    assert_eq!(args, argv(&["check-ref-format", "refs/heads/main"]));
}

#[test]
fn empty_name_is_run() {
    let (args, _) = check_ref_format_request("").unwrap();
    assert_eq!(args, argv(&["check-ref-format", ""]));
}

#[test]
fn dash_inside_the_name_is_run() {
    let (args, _) = check_ref_format_request("refs/heads/-x").unwrap();
    assert_eq!(args, argv(&["check-ref-format", "refs/heads/-x"]));
}

#[test]
fn opts_are_read_only_with_exit_1_an_answer() {
    let (_, opts) = check_ref_format_request("refs/heads/main").unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![1]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn leading_dash_is_refused_naming_it() {
    let err = check_ref_format_request("--branch").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"--branch\""), "{}", err.stderr);
}

#[test]
fn nul_in_name_is_refused_naming_it() {
    let err = check_ref_format_request("refs/heads/a\0b").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("ref name"), "{}", err.stderr);
    assert!(err.stderr.contains("\"refs/heads/a\\0b\""), "{}", err.stderr);
}
