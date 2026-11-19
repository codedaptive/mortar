//! The real git version read wired by shell/run.rs into the floor check, through the built mr binary.
//! These are validation tests because they run the built binary as a child process.

use crate::support::{actual_git, script, shim_path, OwnDir};
use std::fs;
use std::process::{Command, Output};

fn join_with_git_version(version: &str) -> Output {
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
        .args(["join", "--json"])
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
    let output = join_with_git_version("2.45.2");
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "{\"verb\":\"join\",\"ok\":false,\"outcome\":\"refused\",\"exit\":1,\"next\":\"install git 2.46 or newer, then repeat the command\",\"lines\":[\"git 2.45.2 found; mortar needs git 2.46 or newer\"],\"owner\":\"this machine's git\"}\n");
    assert!(!stdout.contains("not-a-folder"));
}

#[test]
fn mr_join_with_git_2_46_is_refused_as_not_a_folder() {
    let output = join_with_git_version("2.46.0");
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"outcome\":\"not-a-folder\""), "{stdout}");
    assert!(stdout.contains("\"next\":\"run mr join inside a folder Mortar made\""), "{stdout}");
    assert!(!stdout.contains("mortar needs git"));
}
