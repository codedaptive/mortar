use crate::read_ref_request::read_ref_request;
use std::ffi::OsString;

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn argv_verifies_the_full_name_after_end_of_options() {
    let (args, _) = read_ref_request("refs/heads/main").unwrap();
    assert_eq!(args, argv(&["show-ref", "--verify", "--end-of-options", "refs/heads/main"]));
}

#[test]
fn name_starting_with_a_dash_stays_after_end_of_options() {
    let (args, _) = read_ref_request("--head").unwrap();
    assert_eq!(args, argv(&["show-ref", "--verify", "--end-of-options", "--head"]));
}

#[test]
fn argv_has_no_quiet_flag() {
    let (args, _) = read_ref_request("HEAD").unwrap();
    assert!(!args.contains(&OsString::from("-q")), "{args:?}");
}

#[test]
fn opts_are_read_only_with_exit_128_an_answer() {
    let (_, opts) = read_ref_request("refs/heads/main").unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![128]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn empty_name_is_refused() {
    let err = read_ref_request("").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn nul_in_name_is_refused_naming_it() {
    let err = read_ref_request("refs/heads/a\0b").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("ref name"), "{}", err.stderr);
    assert!(err.stderr.contains("\"refs/heads/a\\0b\""), "{}", err.stderr);
}
