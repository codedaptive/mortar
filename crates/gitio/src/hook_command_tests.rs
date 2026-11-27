use crate::hook_command::hook_command;
use crate::Git;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

/// The final setting for `key`: None when not touched, Some(None) when removed, Some(Some(v)) when set.
fn env_of(command: &Command, key: &str) -> Option<Option<OsString>> {
    command
        .get_envs()
        .find(|(k, _)| *k == OsStr::new(key))
        .map(|(_, v)| v.map(OsStr::to_os_string))
}

fn set(value: &str) -> Option<Option<OsString>> {
    Some(Some(OsString::from(value)))
}

fn built(git: &Git) -> Command {
    hook_command(
        git,
        Path::new("/repo/.git/hooks/commit-msg"),
        &[OsString::from("/tmp/mortar-message-1-2-3")],
        Path::new("/repo"),
        Path::new("/tmp/mortar-index-1-2-3"),
    )
}

#[test]
fn program_is_the_hook_and_args_are_as_given() {
    let command = built(&Git::new("/repo"));
    assert_eq!(command.get_program(), OsStr::new("/repo/.git/hooks/commit-msg"));
    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(args, vec!["/tmp/mortar-message-1-2-3"]);
}

#[test]
fn current_dir_is_the_folder() {
    assert_eq!(built(&Git::new("/repo")).get_current_dir(), Some(Path::new("/repo")));
}

#[test]
fn each_scrubbed_variable_is_removed_and_the_index_is_set() {
    let command = built(&Git::new("/repo"));
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_PREFIX",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NOGLOB_PATHSPECS",
        "GIT_LITERAL_PATHSPECS",
        "GIT_GLOB_PATHSPECS",
        "GIT_ICASE_PATHSPECS",
    ] {
        assert_eq!(env_of(&command, key), Some(None), "{key} should be removed");
    }
    assert_eq!(env_of(&command, "GIT_INDEX_FILE"), set("/tmp/mortar-index-1-2-3"));
}

#[test]
fn git_env_reaches_the_hook() {
    let git = Git {
        env: vec![(OsString::from("GIT_CONFIG_COUNT"), OsString::from("1"))],
        ..Git::new("/repo")
    };
    assert_eq!(env_of(&built(&git), "GIT_CONFIG_COUNT"), set("1"));
}

#[test]
fn prompt_is_off_and_a_git_env_prompt_wins() {
    assert_eq!(env_of(&built(&Git::new("/repo")), "GIT_TERMINAL_PROMPT"), set("0"));
    let git = Git {
        env: vec![(OsString::from("GIT_TERMINAL_PROMPT"), OsString::from("1"))],
        ..Git::new("/repo")
    };
    assert_eq!(env_of(&built(&git), "GIT_TERMINAL_PROMPT"), set("1"));
}

#[test]
fn editor_is_the_no_op_colon() {
    assert_eq!(env_of(&built(&Git::new("/repo")), "GIT_EDITOR"), set(":"));
}

#[test]
fn index_wins_over_a_git_env_index() {
    let git = Git {
        env: vec![(OsString::from("GIT_INDEX_FILE"), OsString::from("/elsewhere/index"))],
        ..Git::new("/repo")
    };
    assert_eq!(env_of(&built(&git), "GIT_INDEX_FILE"), set("/tmp/mortar-index-1-2-3"));
}
