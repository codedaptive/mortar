use crate::real_index_git::real_index_git;
use gitio::Git;
use std::ffi::OsString;
use std::path::PathBuf;

fn pair(key: &str, value: &str) -> (OsString, OsString) {
    (OsString::from(key), OsString::from(value))
}

fn git(env: Vec<(OsString, OsString)>) -> Git {
    Git { program: PathBuf::from("/usr/bin/git"), repo: PathBuf::from("/w/f"), env }
}

#[test]
fn index_file_is_removed_and_the_others_keep_their_order() {
    let found = real_index_git(&git(vec![
        pair("GIT_AUTHOR_NAME", "bilby"),
        pair("GIT_INDEX_FILE", "/tmp/other-index"),
        pair("LANG", "C"),
    ]));
    assert_eq!(found.env, vec![pair("GIT_AUTHOR_NAME", "bilby"), pair("LANG", "C")]);
}

#[test]
fn every_index_file_entry_is_removed() {
    let found = real_index_git(&git(vec![
        pair("GIT_INDEX_FILE", "/tmp/a"),
        pair("LANG", "C"),
        pair("GIT_INDEX_FILE", "/tmp/b"),
    ]));
    assert_eq!(found.env, vec![pair("LANG", "C")]);
}

#[test]
fn only_the_exact_key_is_removed() {
    let found = real_index_git(&git(vec![pair("GIT_INDEX_FILES", "x"), pair("git_index_file", "y")]));
    assert_eq!(found.env, vec![pair("GIT_INDEX_FILES", "x"), pair("git_index_file", "y")]);
}

#[test]
fn env_without_index_file_is_unchanged() {
    let found = real_index_git(&git(vec![pair("LANG", "C"), pair("GIT_AUTHOR_NAME", "bilby")]));
    assert_eq!(found.env, vec![pair("LANG", "C"), pair("GIT_AUTHOR_NAME", "bilby")]);
}

#[test]
fn program_and_repo_are_carried() {
    let found = real_index_git(&git(vec![pair("GIT_INDEX_FILE", "/tmp/a")]));
    assert_eq!(found.program, PathBuf::from("/usr/bin/git"));
    assert_eq!(found.repo, PathBuf::from("/w/f"));
}
