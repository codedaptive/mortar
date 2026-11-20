//! skip_refill: true when the index has unmerged entries, false for a clean, changed-only or untracked-only folder.

use std::path::PathBuf;

use gitio::StatusPaths;

use super::skip_refill::skip_refill;

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

#[test]
fn conflicted_only_skips_the_refill() {
    let status = StatusPaths {
        conflicted: paths(&["src/lib.rs"]),
        ..StatusPaths::default()
    };
    assert!(skip_refill(&status));
}

#[test]
fn a_clean_folder_is_refilled() {
    assert!(!skip_refill(&StatusPaths::default()));
}

#[test]
fn changed_only_is_refilled() {
    let status = StatusPaths {
        changed: paths(&["src/lib.rs", "README.md"]),
        ..StatusPaths::default()
    };
    assert!(!skip_refill(&status));
}

#[test]
fn untracked_only_is_refilled() {
    let status = StatusPaths {
        untracked: paths(&["notes/new.txt"]),
        ..StatusPaths::default()
    };
    assert!(!skip_refill(&status));
}

#[test]
fn conflicted_with_changed_and_untracked_skips_the_refill() {
    let status = StatusPaths {
        changed: paths(&["src/main.rs"]),
        untracked: paths(&["notes/new.txt"]),
        conflicted: paths(&["src/lib.rs"]),
    };
    assert!(skip_refill(&status));
}
