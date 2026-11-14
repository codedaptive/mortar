use crate::fetch_ref_request::fetch_ref_request;
use std::ffi::OsString;

#[test]
fn argv_forces_src_onto_dst_after_end_of_options() {
    let (args, _) = fetch_ref_request("../remote.git", "refs/heads/main", "refs/x/main").unwrap();
    let want: Vec<OsString> = ["fetch", "--no-tags", "--end-of-options", "../remote.git", "+refs/heads/main:refs/x/main"]
        .iter()
        .map(OsString::from)
        .collect();
    assert_eq!(args, want);
}

#[test]
fn opts_are_a_write_with_only_exit_0_an_answer() {
    let (_, opts) = fetch_ref_request("origin", "refs/heads/main", "refs/remotes/origin/main").unwrap();
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
