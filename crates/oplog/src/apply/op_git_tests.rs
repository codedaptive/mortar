//! op_git: a clone with MORTAR_OP=<op id> as the last env entry, the input's own entries kept before it, and
//! the input untouched.

use std::ffi::OsString;
use std::path::PathBuf;

use gitio::Git;

use super::op_git;

fn pair(key: &str, value: &str) -> (OsString, OsString) {
    (OsString::from(key), OsString::from(value))
}

fn git_with(env: Vec<(OsString, OsString)>) -> Git {
    Git {
        program: PathBuf::from("/usr/bin/git"),
        repo: PathBuf::from("/tmp/some-repo"),
        env,
    }
}

#[test]
fn appends_mortar_op_with_the_op_id_as_the_last_entry() {
    let git = git_with(vec![pair("GIT_AUTHOR_NAME", "a"), pair("LANG", "C")]);
    let op = op_git(&git, "op-123");
    assert_eq!(op.env.last(), Some(&pair("MORTAR_OP", "op-123")));
}

#[test]
fn keeps_the_existing_entries_in_order_before_it() {
    let git = git_with(vec![pair("GIT_AUTHOR_NAME", "a"), pair("LANG", "C")]);
    let op = op_git(&git, "op-123");
    assert_eq!(
        op.env,
        vec![
            pair("GIT_AUTHOR_NAME", "a"),
            pair("LANG", "C"),
            pair("MORTAR_OP", "op-123"),
        ]
    );
}

#[test]
fn leaves_the_input_unchanged() {
    let git = git_with(vec![pair("GIT_AUTHOR_NAME", "a"), pair("LANG", "C")]);
    op_git(&git, "op-123");
    assert_eq!(git.env, vec![pair("GIT_AUTHOR_NAME", "a"), pair("LANG", "C")]);
}

#[test]
fn keeps_program_and_repo() {
    let git = git_with(vec![pair("LANG", "C")]);
    let op = op_git(&git, "op-123");
    assert_eq!(op.program, PathBuf::from("/usr/bin/git"));
    assert_eq!(op.repo, PathBuf::from("/tmp/some-repo"));
}

#[test]
fn empty_env_gives_exactly_the_one_entry() {
    let op = op_git(&git_with(Vec::new()), "op-7");
    assert_eq!(op.env, vec![pair("MORTAR_OP", "op-7")]);
}
