use crate::is_ancestor_request::is_ancestor_request;
use crate::Oid;
use std::ffi::OsString;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn argv_puts_both_commits_after_end_of_options_in_order() {
    let (args, _) = is_ancestor_request(
        &oid("726080b6b018f93a1bbce14e3537e998ff5f62bd"),
        &oid("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
    );
    let want: Vec<OsString> = [
        "merge-base",
        "--is-ancestor",
        "--end-of-options",
        "726080b6b018f93a1bbce14e3537e998ff5f62bd",
        "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, want);
}

#[test]
fn opts_are_read_only_with_exit_1_an_answer() {
    let (_, opts) = is_ancestor_request(
        &oid("726080b6b018f93a1bbce14e3537e998ff5f62bd"),
        &oid("6269f0e88f79b3eada64287692f6d3f03196c4a7"),
    );
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![1]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
