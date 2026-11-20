//! Real-I/O integration joins for the shared testkit; nothing here measures time.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use testkit::{test_child, write_git_shim, TestRepo};

#[test]
fn repositories_are_initialized_independently_and_removed_on_drop_or_close() {
    // Joins TestRepo::new -> real git init -> TempDir drop / TestRepo::close.
    let first = TestRepo::new().unwrap();
    let second = TestRepo::new().unwrap();
    assert_ne!(first.path, second.path);
    assert_eq!(fs::read(first.path.join(".git/HEAD")).unwrap(), b"ref: refs/heads/main\n");
    let output = Command::new("git").arg("-C").arg(&first.path)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["rev-parse", "--show-toplevel"]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(std::path::Path::new(String::from_utf8(output.stdout).unwrap().trim_end()), first.path.canonicalize().unwrap());
    let first_path = first.path.clone();
    let second_path = second.path.clone();
    drop(first);
    assert!(!first_path.exists());
    assert!(second_path.join(".git/HEAD").is_file());
    second.close().unwrap();
    assert!(!second_path.exists());
}

#[test]
fn shared_child_selector_runs_the_requested_worker_with_its_environment() {
    // Joins test_child -> this executable's ignored worker -> an independently observed marker.
    let repo = TestRepo::new().unwrap();
    let marker = repo.path.join("child-marker");
    let output = test_child("child_worker").unwrap()
        .env("MORTAR_TESTKIT_CHILD_MARKER", &marker).output().unwrap();
    assert!(output.status.success(), "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read(marker).unwrap(), b"child-ran\n");
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[test]
fn executable_shim_delegates_arguments_to_real_git_in_the_owned_repository() {
    // Joins write_git_shim -> executable permissions -> real git repository discovery.
    let repo = TestRepo::new().unwrap();
    let shim = repo.path.join("git-shim.sh");
    write_git_shim(&shim, b"#!/bin/sh\nexec git \"$@\"\n").unwrap();
    assert_eq!(fs::metadata(&shim).unwrap().permissions().mode() & 0o777, 0o700);
    let output = Command::new(&shim).arg("-C").arg(&repo.path)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["rev-parse", "--show-toplevel"]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(std::path::Path::new(String::from_utf8(output.stdout).unwrap().trim_end()), repo.path.canonicalize().unwrap());
}

#[test]
#[ignore = "subprocess entry point; the parent verifies its independent filesystem outcome"]
fn child_worker() {
    let marker = std::env::var_os("MORTAR_TESTKIT_CHILD_MARKER").unwrap();
    fs::write(marker, b"child-ran\n").unwrap();
}
