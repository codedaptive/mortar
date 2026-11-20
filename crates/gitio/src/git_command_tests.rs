use crate::git_command::git_command;
use crate::{Git, RunOpts};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;

fn os(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

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

#[test]
fn program_is_the_git_value_program() {
    let git = Git {
        program: PathBuf::from("/opt/git/bin/git"),
        repo: PathBuf::from("/tmp/repo"),
        env: Vec::new(),
    };
    let command = git_command(&git, &[], &RunOpts::default());
    assert_eq!(command.get_program(), OsStr::new("/opt/git/bin/git"));
}

#[test]
fn dash_c_names_the_repo_then_the_args() {
    let command = git_command(&Git::new("/tmp/repo"), &os(&["status", "--porcelain"]), &RunOpts::default());
    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(args, vec!["-C", "/tmp/repo", "status", "--porcelain"]);
}

#[test]
fn opts_cwd_replaces_the_repo() {
    let opts = RunOpts {
        cwd: Some(PathBuf::from("/tmp/other")),
        ..RunOpts::default()
    };
    let command = git_command(&Git::new("/tmp/repo"), &os(&["log"]), &opts);
    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(args, vec!["-C", "/tmp/other", "log"]);
}

#[test]
fn args_stay_one_each_with_spaces_and_semicolons() {
    let command = git_command(
        &Git::new("/tmp/repo"),
        &os(&["commit", "-m", "two words", "a; rm -rf b"]),
        &RunOpts::default(),
    );
    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(args, vec!["-C", "/tmp/repo", "commit", "-m", "two words", "a; rm -rf b"]);
}

#[test]
fn each_hook_variable_is_removed() {
    let command = git_command(&Git::new("/tmp/repo"), &[], &RunOpts::default());
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_PREFIX",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        assert_eq!(env_of(&command, key), Some(None), "{key} should be removed");
    }
}

#[test]
fn each_pathspec_mode_variable_is_removed() {
    let command = git_command(&Git::new("/tmp/repo"), &[], &RunOpts::default());
    for key in [
        "GIT_NOGLOB_PATHSPECS",
        "GIT_LITERAL_PATHSPECS",
        "GIT_GLOB_PATHSPECS",
        "GIT_ICASE_PATHSPECS",
    ] {
        assert_eq!(env_of(&command, key), Some(None), "{key} should be removed");
    }
}

#[test]
fn opts_env_pathspec_mode_wins_over_the_scrub() {
    for key in [
        "GIT_NOGLOB_PATHSPECS",
        "GIT_LITERAL_PATHSPECS",
        "GIT_GLOB_PATHSPECS",
        "GIT_ICASE_PATHSPECS",
    ] {
        let opts = RunOpts {
            env: vec![(OsString::from(key), OsString::from("1"))],
            ..RunOpts::default()
        };
        let command = git_command(&Git::new("/tmp/repo"), &[], &opts);
        assert_eq!(env_of(&command, key), set("1"), "{key} from opts.env should survive");
    }
}

#[test]
fn git_env_pathspec_mode_wins_over_the_scrub() {
    let git = Git {
        env: vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1"))],
        ..Git::new("/tmp/repo")
    };
    let command = git_command(&git, &[], &RunOpts::default());
    assert_eq!(env_of(&command, "GIT_LITERAL_PATHSPECS"), set("1"));
}

#[test]
fn locale_and_prompt_are_fixed() {
    let command = git_command(&Git::new("/tmp/repo"), &[], &RunOpts::default());
    assert_eq!(env_of(&command, "LC_ALL"), set("C"));
    assert_eq!(env_of(&command, "GIT_TERMINAL_PROMPT"), set("0"));
}

#[test]
fn a_write_run_leaves_optional_locks_alone() {
    let command = git_command(&Git::new("/tmp/repo"), &[], &RunOpts::default());
    assert_eq!(env_of(&command, "GIT_OPTIONAL_LOCKS"), None);
}

#[test]
fn a_read_run_turns_optional_locks_off() {
    let opts = RunOpts {
        read_only: true,
        ..RunOpts::default()
    };
    let command = git_command(&Git::new("/tmp/repo"), &[], &opts);
    assert_eq!(env_of(&command, "GIT_OPTIONAL_LOCKS"), set("0"));
}

#[test]
fn git_env_wins_over_the_scrub() {
    let git = Git {
        env: vec![(OsString::from("GIT_INDEX_FILE"), OsString::from("x"))],
        ..Git::new("/tmp/repo")
    };
    let command = git_command(&git, &[], &RunOpts::default());
    assert_eq!(env_of(&command, "GIT_INDEX_FILE"), set("x"));
}

#[test]
fn git_env_wins_over_the_fixed_settings() {
    let git = Git {
        env: vec![(OsString::from("LC_ALL"), OsString::from("en_US.UTF-8"))],
        ..Git::new("/tmp/repo")
    };
    let command = git_command(&git, &[], &RunOpts::default());
    assert_eq!(env_of(&command, "LC_ALL"), set("en_US.UTF-8"));
}

#[test]
fn opts_env_wins_over_git_env() {
    let git = Git {
        env: vec![(OsString::from("GIT_AUTHOR_NAME"), OsString::from("repo-wide"))],
        ..Git::new("/tmp/repo")
    };
    let opts = RunOpts {
        env: vec![(OsString::from("GIT_AUTHOR_NAME"), OsString::from("this-run"))],
        ..RunOpts::default()
    };
    let command = git_command(&git, &[], &opts);
    assert_eq!(env_of(&command, "GIT_AUTHOR_NAME"), set("this-run"));
}

fn read_opts() -> RunOpts {
    RunOpts {
        read_only: true,
        ..RunOpts::default()
    }
}

fn pairs(items: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    items.iter().map(|(k, v)| (OsString::from(k), OsString::from(v))).collect()
}

#[test]
fn a_read_run_keeps_the_argv_plain() {
    let command = git_command(&Git::new("/tmp/repo"), &os(&["diff", "HEAD"]), &read_opts());
    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(args, vec!["-C", "/tmp/repo", "diff", "HEAD"]);
}

#[test]
fn a_read_run_ends_its_config_parameters_with_no_refresh() {
    let command = git_command(&Git::new("/tmp/repo"), &[], &read_opts());
    let value = env_of(&command, "GIT_CONFIG_PARAMETERS").flatten().expect("set on a read run");
    assert!(
        value.to_string_lossy().ends_with("'diff.autoRefreshIndex'='false'"),
        "{value:?}"
    );
}

#[test]
fn git_env_config_parameters_are_kept_before_no_refresh() {
    let git = Git {
        env: pairs(&[("GIT_CONFIG_PARAMETERS", "'core.x'='y'")]),
        ..Git::new("/tmp/repo")
    };
    let command = git_command(&git, &[], &read_opts());
    assert_eq!(
        env_of(&command, "GIT_CONFIG_PARAMETERS"),
        set("'core.x'='y' 'diff.autoRefreshIndex'='false'")
    );
}

#[test]
fn opts_env_config_parameters_beat_git_envs_as_the_base() {
    let git = Git {
        env: pairs(&[("GIT_CONFIG_PARAMETERS", "'core.x'='y'")]),
        ..Git::new("/tmp/repo")
    };
    let opts = RunOpts {
        env: pairs(&[("GIT_CONFIG_PARAMETERS", "'core.z'='w'")]),
        ..read_opts()
    };
    let command = git_command(&git, &[], &opts);
    assert_eq!(
        env_of(&command, "GIT_CONFIG_PARAMETERS"),
        set("'core.z'='w' 'diff.autoRefreshIndex'='false'")
    );
}

#[test]
fn opts_env_config_count_stays_and_no_refresh_is_still_set() {
    let opts = RunOpts {
        env: pairs(&[
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "diff.autoRefreshIndex"),
            ("GIT_CONFIG_VALUE_0", "true"),
            ("GIT_CONFIG_PARAMETERS", ""),
        ]),
        ..read_opts()
    };
    let command = git_command(&Git::new("/tmp/repo"), &[], &opts);
    assert_eq!(env_of(&command, "GIT_CONFIG_COUNT"), set("1"));
    assert_eq!(env_of(&command, "GIT_CONFIG_KEY_0"), set("diff.autoRefreshIndex"));
    assert_eq!(env_of(&command, "GIT_CONFIG_VALUE_0"), set("true"));
    assert_eq!(env_of(&command, "GIT_CONFIG_PARAMETERS"), set("'diff.autoRefreshIndex'='false'"));
}

#[test]
fn a_write_run_leaves_config_parameters_alone() {
    let command = git_command(&Git::new("/tmp/repo"), &os(&["diff", "HEAD"]), &RunOpts::default());
    assert_eq!(env_of(&command, "GIT_CONFIG_PARAMETERS"), None);
}
