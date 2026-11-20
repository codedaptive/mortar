use crate::Git;
use std::ffi::OsString;
use std::path::PathBuf;

#[test]
fn clone_keeps_program_repo_and_env() {
    let git = Git {
        program: PathBuf::from("/opt/git/bin/git"),
        repo: PathBuf::from("/tmp/repo"),
        env: vec![(OsString::from("GIT_INDEX_FILE"), OsString::from("/tmp/idx"))],
    };
    let copy = git.clone();
    assert_eq!(copy.program, PathBuf::from("/opt/git/bin/git"));
    assert_eq!(copy.repo, PathBuf::from("/tmp/repo"));
    assert_eq!(
        copy.env,
        vec![(OsString::from("GIT_INDEX_FILE"), OsString::from("/tmp/idx"))]
    );
}
