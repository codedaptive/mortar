use crate::Git;
use std::path::PathBuf;

#[test]
fn new_uses_git_on_path_with_no_env() {
    let git = Git::new("/tmp/repo");
    assert_eq!(git.program, PathBuf::from("git"));
    assert_eq!(git.repo, PathBuf::from("/tmp/repo"));
    assert!(git.env.is_empty());
}

#[test]
fn new_takes_a_pathbuf() {
    let git = Git::new(PathBuf::from("relative/dir"));
    assert_eq!(git.repo, PathBuf::from("relative/dir"));
}
