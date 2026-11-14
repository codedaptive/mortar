use crate::hooks_path_request::hooks_path_request;
use std::ffi::OsString;

#[test]
fn argv_asks_for_the_absolute_hooks_git_path() {
    let (args, _) = hooks_path_request();
    let want: Vec<OsString> =
        ["rev-parse", "--path-format=absolute", "--git-path", "hooks"].map(OsString::from).to_vec();
    assert_eq!(args, want);
}

#[test]
fn opts_are_read_only_in_the_repo_with_no_answer_codes() {
    let (_, opts) = hooks_path_request();
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}
