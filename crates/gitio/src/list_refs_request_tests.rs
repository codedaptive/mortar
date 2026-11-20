use crate::list_refs_request::list_refs_request;
use std::ffi::OsString;

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn prefix_follows_end_of_options() {
    let (args, _) = list_refs_request("refs/heads/wt").unwrap();
    assert_eq!(
        args,
        argv(&["for-each-ref", "--format=%(objectname) %(refname)", "--end-of-options", "refs/heads/wt"])
    );
}

#[test]
fn empty_prefix_is_left_out() {
    let (args, _) = list_refs_request("").unwrap();
    assert_eq!(args, argv(&["for-each-ref", "--format=%(objectname) %(refname)", "--end-of-options"]));
}

#[test]
fn prefix_starting_with_a_dash_stays_after_end_of_options() {
    let (args, _) = list_refs_request("--sort=x").unwrap();
    assert_eq!(args, argv(&["for-each-ref", "--format=%(objectname) %(refname)", "--end-of-options", "--sort=x"]));
}

#[test]
fn opts_are_read_only_with_only_exit_0() {
    let (_, opts) = list_refs_request("refs/heads/").unwrap();
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn nul_in_prefix_is_refused_naming_it() {
    let err = list_refs_request("refs/\0heads").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"refs/\\0heads\""), "{}", err.stderr);
}
