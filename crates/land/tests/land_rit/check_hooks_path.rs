//! RITs: Land check_hooks_path -> Gitio Git::run -> real Git config, including --type=path expansion;
//! a real config read failure also joins HooksPathError::Display/source to GitError::Display.
//! Each TestRepo owns its config and HOME and removes them on drop; no check installs hooks or writes guard.json.

use gitio::Git;
use land::{check_hooks_path, HooksPathError};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use testkit::TestRepo;

fn repo() -> (TestRepo, Git) {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    let git = Git { program: "git".into(), repo: root.clone(), env: vec![
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_COUNT".into(), "0".into()), ("GIT_CONFIG_PARAMETERS".into(), "".into()),
        ("HOME".into(), root.into_os_string()),
    ] };
    (repo, git)
}

fn configure(git: &Git, value: &str) {
    let output = Command::new("git").arg("-C").arg(&git.repo).args(["config", "core.hooksPath", value])
        .env_clear().envs(git.env.iter().cloned()).env("PATH", std::env::var_os("PATH").unwrap())
        .env("LC_ALL", "C").output().unwrap();
    assert!(output.status.success(), "git config: {output:?}");
}

fn checked_without_writes(git: &Git) -> Result<(), HooksPathError> {
    let config = git.repo.join(".git/config");
    let before = fs::read(&config).unwrap();
    let result = check_hooks_path(git);
    assert_eq!(fs::read(config).unwrap(), before);
    assert!(!git.repo.join(".git/mortar").exists());
    assert!(!git.repo.join(".git/hooks/reference-transaction").exists());
    assert!(!git.repo.join(".git/hooks/pre-push").exists());
    result
}

#[test]
fn rit_check_hooks_path_unset_passes_without_writes() {
    let (_repo, git) = repo();
    checked_without_writes(&git).unwrap();
}

#[test]
fn rit_check_hooks_path_absolute_passes_without_creating_the_hooks_directory() {
    let (_repo, git) = repo();
    let hooks = git.repo.join("shared hooks");
    configure(&git, hooks.to_str().unwrap());
    checked_without_writes(&git).unwrap();
    assert!(!hooks.exists());
}

#[test]
fn rit_check_hooks_path_relative_is_refused_with_its_value_without_writes() {
    let (_repo, git) = repo();
    configure(&git, "myhooks");
    let error = checked_without_writes(&git).unwrap_err();
    assert!(matches!(&error, HooksPathError::Relative { value } if value == Path::new("myhooks")), "{error:?}");
    assert_eq!(error.to_string(), "core.hooksPath is \"myhooks\", not an absolute path: git resolves a relative hooks path in each folder, so one install cannot guard them all");
    assert!(error.source().is_none());
    assert!(!git.repo.join("myhooks").exists());
}

#[test]
fn rit_check_hooks_path_empty_fails_closed_without_writes() {
    let (_repo, git) = repo();
    configure(&git, "");
    let error = checked_without_writes(&git).unwrap_err();
    assert!(matches!(&error, HooksPathError::Relative { value } if value == &PathBuf::new()), "{error:?}");
}

#[test]
fn rit_check_hooks_path_tilde_value_passes_without_creating_hooks() {
    let (_repo, git) = repo();
    configure(&git, "~/shared hooks");
    checked_without_writes(&git).unwrap();
    assert!(!git.repo.join("shared hooks").exists());
}

#[test]
fn rit_check_hooks_path_git_failure_is_not_unset_and_keeps_git_display_and_source() {
    let (_repo, git) = repo();
    fs::write(git.repo.join(".git/config"), "[broken\n").unwrap();
    let error = checked_without_writes(&git).unwrap_err();
    let HooksPathError::Git(inner) = &error else { panic!("expected Git failure, got {error:?}") };
    assert_eq!(inner.code, Some(128));
    assert_eq!(inner.args, ["config", "--type=path", "--get", "core.hooksPath"]);
    assert_eq!(error.to_string(), "git could not read core.hooksPath: git config --type=path --get core.hooksPath: exit code 128: fatal: bad config line 1 in file .git/config");
    assert_eq!(error.source().unwrap().to_string(), "git config --type=path --get core.hooksPath: exit code 128: fatal: bad config line 1 in file .git/config");
}
