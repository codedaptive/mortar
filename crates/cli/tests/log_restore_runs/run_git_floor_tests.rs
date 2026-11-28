//! The real git version read wired by shell/run.rs into the floor check, through the built mr binary.
//! They run the built binary as a child process.

use crate::support::{actual_git, script, shim_path, OwnDir};
use std::fs;
use std::process::{Command, Output};

fn mr_with_git_version(version: &str, words: &[&str]) -> Output {
    let real_git = actual_git();
    let real_git = real_git.to_str().expect("real git path is not UTF-8").replace('\'', "'\\''");

    let temp = OwnDir::new();
    let root = &temp.0;
    let bin = root.join("bin");
    let work = root.join("work");
    fs::create_dir(&bin).unwrap();
    fs::create_dir(&work).unwrap();
    script(
        &bin.join("git"),
        format!(
            "#!/bin/sh\nif {{ [ \"$1\" = -C ] && [ \"$3\" = version ]; }} || [ \"$1\" = --version ] || [ \"$1\" = version ]; then\n  printf 'git version {version}\\n'\n  exit 0\nfi\nexec '{real_git}' \"$@\"\n"
        )
        .as_bytes(),
    );
    Command::new(env!("CARGO_BIN_EXE_mr"))
        .args(words)
        .current_dir(work)
        .env_clear()
        .env("PATH", shim_path(&bin))
        .env("HOME", root)
        .env("TMPDIR", root)
        .env("GIT_CEILING_DIRECTORIES", root)
        .output()
        .expect("could not run the built mr binary")
}

#[test]
fn mr_join_with_git_2_45_is_refused_by_the_floor_before_any_folder_read() {
    let output = mr_with_git_version("2.45.2", &["join", "--json"]);
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "{\"verb\":\"join\",\"ok\":false,\"outcome\":\"refused\",\"exit\":3,\"next\":\"install git 2.46 or newer, then repeat the command\",\"lines\":[\"git 2.45.2 found; mortar needs git 2.46 or newer\"],\"owner\":\"this machine's git\"}\n");
    assert!(!stdout.contains("not-a-repository"));
}

#[test]
fn mr_join_with_git_2_46_outside_a_repository_is_usage_not_a_repository() {
    for words in [vec!["join", "--json"], vec!["join"]] {
        let output = mr_with_git_version("2.46.0", &words);
        assert_eq!(output.status.code(), Some(2), "{words:?}: {output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        if words.contains(&"--json") { assert!(stdout.contains("\"outcome\":\"not-a-repository\""), "{stdout}"); }
        assert!(stdout.contains("run mr join inside the git repository"), "{stdout}");
        assert!(!stdout.contains("mortar needs git"));
    }
}

#[test]
fn mr_log_outside_a_repository_is_usage_not_a_repository() {
    for words in [vec!["log", "--json"], vec!["log"]] {
        let output = mr_with_git_version("2.46.0", &words);
        assert_eq!(output.status.code(), Some(2), "{words:?}: {output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        if words.contains(&"--json") { assert!(stdout.contains("\"outcome\":\"not-a-repository\""), "{stdout}"); }
        assert!(stdout.contains("whoever sets up Mortar in this repository"), "{stdout}");
        assert!(stdout.contains("run mr log inside the git repository"), "{stdout}");
    }
}
