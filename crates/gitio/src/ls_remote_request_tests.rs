use crate::ls_remote_request::ls_remote_request;
use std::ffi::OsString;

fn argv(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

#[test]
fn argv_puts_remote_and_patterns_after_end_of_options_in_order() {
    let patterns = ["refs/heads/main".to_string(), "v1".to_string()];
    let (args, _) = ls_remote_request("../remote.git", &patterns).unwrap();
    assert_eq!(args, argv(&["ls-remote", "--end-of-options", "../remote.git", "refs/heads/main", "v1"]));
}

#[test]
fn argv_without_patterns_ends_at_the_remote() {
    let (args, _) = ls_remote_request("origin", &[]).unwrap();
    assert_eq!(args, argv(&["ls-remote", "--end-of-options", "origin"]));
}

#[test]
fn opts_are_read_only_with_only_exit_0_an_answer() {
    let (_, opts) = ls_remote_request("origin", &["main".to_string()]).unwrap();
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
