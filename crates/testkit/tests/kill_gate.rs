//! Real process/pipe joins for the crash gate; no timing bounds, sleeps, retries or stress loops.

use std::fs;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim, write_git_shim, TestRepo};

#[test]
fn before_git_boundary_requires_reaped_worker() {
    assert_boundary("before");
}

#[test]
fn after_git_boundary_requires_reaped_worker() {
    assert_boundary("after");
}

fn assert_boundary(stage: &str) {
    // Joins write_git_kill_shim -> real git update-ref -> wait/release -> kernel-observed ack.
    // A prematurely released live worker or zombie answers kill -0, so the helper refuses it.
    let repo = TestRepo::new().unwrap();
    write_git_kill_shim(&repo.path.join("git-kill.sh")).unwrap();
    let mut command = test_child("git_worker").unwrap();
    command.env("TESTKIT_WORKER_REPO", &repo.path).env("MORTAR_TEST_CRASH", stage)
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null");
    let output = run_with_kill_gate(&mut command, &repo.path).unwrap();
    assert_eq!(output.status.signal(), Some(9), "{stage}: {output:?}");
    assert!(!repo.path.join("worker-returned-from-wrapper").exists(), "{stage}: worker resumed after wrapper exit");
    assert!(!fs::read_dir(&repo.path).unwrap().any(|entry|
        entry.unwrap().file_name().to_string_lossy().starts_with("kill-gate-")));
}

#[test]
fn early_normal_exit_returns_its_output_without_waiting_for_an_unentered_gate() {
    // Joins run_with_kill_gate -> actual child exit and captured stdout/stderr, with no shim.
    let repo = TestRepo::new().unwrap();
    let mut command = test_child("ordinary_worker").unwrap();
    let output = run_with_kill_gate(&mut command, &repo.path).unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ordinary stdout"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ordinary stderr"));
    assert!(!fs::read_dir(&repo.path).unwrap().any(|entry|
        entry.unwrap().file_name().to_string_lossy().starts_with("kill-gate-")));
}

#[test]
fn killed_worker_without_a_fifo_reader_returns_an_unarmed_boundary_error() {
    // Joins a shim exiting 98 without opening a FIFO -> killed worker -> gate error/cleanup.
    // Verify the returned diagnostic, not an elapsed-time bound; no reader is ever created.
    for marker in ["missing", "invalid"] {
        let repo = TestRepo::new().unwrap();
        write_git_shim(&repo.path.join("unarmed-shim.sh"), b"#!/bin/sh\nexit 98\n").unwrap();
        let mut command = test_child("unarmed_worker").unwrap();
        command.env("TESTKIT_WORKER_REPO", &repo.path).env("TESTKIT_UNARMED_MARKER", marker);
        let error = run_with_kill_gate(&mut command, &repo.path).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("worker died without a valid armed crash boundary"), "{marker}: {message}");
        assert!(message.contains("shim exited 98 without a FIFO reader"), "{marker}: {message}");
        assert!(!fs::read_dir(&repo.path).unwrap().any(|entry|
            entry.unwrap().file_name().to_string_lossy().starts_with("kill-gate-")));
    }
}

#[test]
#[ignore = "subprocess entry point; parent verifies an error after shim exit without an armed boundary"]
fn unarmed_worker() {
    use std::os::unix::process::CommandExt;

    let repo = std::path::PathBuf::from(std::env::var_os("TESTKIT_WORKER_REPO").unwrap());
    let status = Command::new(repo.join("unarmed-shim.sh")).status().unwrap();
    assert_eq!(status.code(), Some(98));
    if std::env::var("TESTKIT_UNARMED_MARKER").unwrap() == "invalid" {
        fs::write(std::env::var_os("TESTKIT_GATE_ARMED").unwrap(), b"not an armed boundary\n").unwrap();
    }
    println!("shim exited 98 without a FIFO reader");
    std::io::stdout().flush().unwrap();
    // Replace this worker in place, retaining its PID, with a shell that kills itself.
    // No descendant stays alive holding a FIFO or its parent's output pipes.
    let error = Command::new("sh").args(["-c", "kill -KILL $$"]).exec();
    panic!("could not replace worker with self-kill: {error}");
}

#[test]
#[ignore = "subprocess entry point; real git wrapper is held until the parent has reaped this process"]
fn git_worker() {
    let repo = std::path::PathBuf::from(std::env::var_os("TESTKIT_WORKER_REPO").unwrap());
    let mut command = Command::new(repo.join("git-kill.sh"));
    command.arg("-C").arg(&repo).args(["update-ref", "--stdin", "-z"])
        .env("MORTAR_TEST_WORKER_PID", std::process::id().to_string())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX",
        "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES"] {
        command.env_remove(name);
    }
    let mut wrapper = command.spawn().unwrap();
    wrapper.stdin.take().unwrap().write_all(b"start\0prepare\0commit\0").unwrap();
    wrapper.wait_with_output().unwrap();
    // This is forbidden by the gate's contract, not something the parent waits to see.
    fs::write(repo.join("worker-returned-from-wrapper"), b"incorrectly resumed\n").unwrap();
}

#[test]
#[ignore = "subprocess entry point; parent checks normal exit and both output streams"]
fn ordinary_worker() {
    println!("ordinary stdout");
    eprintln!("ordinary stderr");
}
