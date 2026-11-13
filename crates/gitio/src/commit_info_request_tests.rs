use crate::commit_info_request::commit_info_request;
use crate::Oid;
use std::ffi::OsString;

#[test]
fn argv_is_one_nul_record_of_id_tree_parents_and_message() {
    let (args, _) = commit_info_request(&Oid::parse("726080b6b018f93a1bbce14e3537e998ff5f62bd").unwrap());
    let want: Vec<OsString> = [
        "log",
        "-1",
        "-z",
        "--no-show-signature",
        "--encoding=UTF-8",
        "--format=%H%x00%T%x00%P%x00%B",
        "--end-of-options",
        "726080b6b018f93a1bbce14e3537e998ff5f62bd",
        "--",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, want);
}

#[test]
fn opts_are_read_only_with_no_answer_codes() {
    let (_, opts) = commit_info_request(
        &Oid::parse("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789").unwrap(),
    );
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
