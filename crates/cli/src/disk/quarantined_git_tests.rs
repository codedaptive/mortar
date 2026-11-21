//! quarantined_git: the caller's program, repo and env pairs are kept, and the two object pairs follow them in order.
//! No git runs and nothing is made on disk.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use gitio::Git;

use super::quarantined_git;

fn pair(key: &str, value: &str) -> (OsString, OsString) {
    (OsString::from(key), OsString::from(value))
}

fn git(env: Vec<(OsString, OsString)>) -> Git {
    Git { program: PathBuf::from("git"), repo: PathBuf::from("/r/folder"), env }
}

fn quarantined(env: Vec<(OsString, OsString)>) -> Git {
    quarantined_git(&git(env), Path::new("/tmp/q"), Path::new("/r/.git/objects"))
}

#[test]
fn the_program_and_the_repo_are_kept() {
    let seen = quarantined(vec![pair("GIT_AUTHOR_NAME", "someone")]);
    assert_eq!(seen.program, PathBuf::from("git"));
    assert_eq!(seen.repo, PathBuf::from("/r/folder"));
}

#[test]
fn the_env_is_the_callers_pair_then_the_object_dir_then_the_alternate() {
    assert_eq!(
        quarantined(vec![pair("GIT_AUTHOR_NAME", "someone")]).env,
        vec![
            pair("GIT_AUTHOR_NAME", "someone"),
            pair("GIT_OBJECT_DIRECTORY", "/tmp/q"),
            pair("GIT_ALTERNATE_OBJECT_DIRECTORIES", "/r/.git/objects"),
        ]
    );
}

#[test]
fn two_caller_pairs_are_both_kept_first_in_their_order() {
    assert_eq!(
        quarantined(vec![pair("GIT_AUTHOR_NAME", "someone"), pair("GIT_INDEX_FILE", "/tmp/index")]).env,
        vec![
            pair("GIT_AUTHOR_NAME", "someone"),
            pair("GIT_INDEX_FILE", "/tmp/index"),
            pair("GIT_OBJECT_DIRECTORY", "/tmp/q"),
            pair("GIT_ALTERNATE_OBJECT_DIRECTORIES", "/r/.git/objects"),
        ]
    );
}
