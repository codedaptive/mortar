//! Guard failure texts: each test pins the whole Display text, in land's own words around the values the test passes,
//! except: the Git arm is land's prefix then gitio's text; the Io arm is the path then std's io::Error text.

use std::io;
use std::path::PathBuf;

use gitio::GitError;

use super::GuardError;

fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}

#[test]
fn git_text_is_git_failed_then_gitios_command_exit_code_and_stderr() {
    assert_eq!(GuardError::Git(git_error()).to_string(), "git failed: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn io_text_is_the_path_then_the_io_error_text() {
    let err = GuardError::Io { path: PathBuf::from("/r/.git/hooks/pre-push"), error: io::Error::other("disk") };
    assert_eq!(err.to_string(), "/r/.git/hooks/pre-push: disk");
}

#[test]
fn foreign_hook_text_is_the_file_why_never_overwritten_then_what_to_do() {
    let err = GuardError::ForeignHook { path: PathBuf::from("/r/.git/hooks/pre-push") };
    assert_eq!(
        err.to_string(),
        "/r/.git/hooks/pre-push is a hook Mortar did not write, and Mortar never overwrites one: move it aside or fold \
         it in by hand, then install the guard again"
    );
}

#[test]
fn bad_config_text_is_the_file_not_a_guard_config_then_why() {
    let err = GuardError::BadConfig {
        path: PathBuf::from("/r/.git/mortar/guard.json"),
        why: "key \"remote\" appears twice".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "/r/.git/mortar/guard.json is not a Mortar guard config: key \"remote\" appears twice"
    );
}
