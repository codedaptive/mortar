//! attach_head_request: `symbolic-ref HEAD refs/heads/<branch>`, run in the folder, as a write command.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::attach_head_request::attach_head_request;

#[test]
fn points_head_at_the_full_branch_ref() {
    let (args, _) = attach_head_request(Path::new("/wt/req"), "req/folders/rf1");
    let want: Vec<OsString> = ["symbolic-ref", "HEAD", "refs/heads/req/folders/rf1"]
        .iter()
        .map(OsString::from)
        .collect();
    assert_eq!(args, want);
}

#[test]
fn runs_in_the_folder_as_a_write() {
    let (_, opts) = attach_head_request(Path::new("/wt/req"), "zone/folders");
    assert_eq!(opts.cwd, Some(PathBuf::from("/wt/req")));
    assert!(!opts.read_only);
    assert_eq!(opts.stdin, None);
    assert!(opts.answer_codes.is_empty());
}
