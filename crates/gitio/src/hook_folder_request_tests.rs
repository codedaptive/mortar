use crate::hook_folder_request::hook_folder_request;
use std::ffi::OsString;

#[test]
fn argv_asks_whether_the_repository_is_bare() {
    let (args, _) = hook_folder_request();
    let want: Vec<OsString> = ["rev-parse", "--is-bare-repository"].map(OsString::from).to_vec();
    assert_eq!(args, want);
}

#[test]
fn opts_are_read_only_in_the_repo() {
    let (_, opts) = hook_folder_request();
    assert!(opts.read_only);
    assert_eq!(opts.cwd, None);
}
