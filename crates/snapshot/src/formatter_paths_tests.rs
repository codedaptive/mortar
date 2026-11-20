use crate::formatter_paths::formatter_paths;
use gitio::StatusPaths;
use std::collections::HashSet;
use std::path::PathBuf;

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

fn set(list: &[&str]) -> HashSet<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

fn status(changed: &[&str], untracked: &[&str], conflicted: &[&str]) -> StatusPaths {
    StatusPaths { changed: paths(changed), untracked: paths(untracked), conflicted: paths(conflicted) }
}

#[test]
fn changed_then_untracked_in_order() {
    let status = status(&["src/a.rs", "src/b.rs"], &["new.rs"], &[]);
    let all = set(&["src/a.rs", "src/b.rs", "new.rs"]);
    assert_eq!(formatter_paths(&status, &all, &all), paths(&["src/a.rs", "src/b.rs", "new.rs"]));
}

#[test]
fn a_deleted_path_is_not_passed() {
    let status = status(&["gone.rs", "src/a.rs"], &[], &[]);
    let saved = set(&["gone.rs", "src/a.rs"]);
    assert_eq!(formatter_paths(&status, &saved, &set(&["src/a.rs"])), paths(&["src/a.rs"]));
}

#[test]
fn a_rename_passes_only_its_new_side() {
    let status = status(&["new_name.rs", "old_name.rs"], &[], &[]);
    let saved = set(&["new_name.rs", "old_name.rs"]);
    assert_eq!(formatter_paths(&status, &saved, &set(&["new_name.rs"])), paths(&["new_name.rs"]));
}

#[test]
fn a_path_the_snapshot_excludes_is_not_passed() {
    let status = status(&[], &["target/debug/x.rs", "src/c.rs"], &[]);
    let files = set(&["target/debug/x.rs", "src/c.rs"]);
    assert_eq!(formatter_paths(&status, &set(&["src/c.rs"]), &files), paths(&["src/c.rs"]));
}

#[test]
fn caller_excluded_and_refused_paths_are_not_passed() {
    let status = status(&["vendor/lib.rs"], &["assets/big.bin", "src/c.rs"], &[]);
    let files = set(&["vendor/lib.rs", "assets/big.bin", "src/c.rs"]);
    assert_eq!(formatter_paths(&status, &set(&["src/c.rs"]), &files), paths(&["src/c.rs"]));
}

#[test]
fn a_conflicted_path_is_not_passed() {
    let status = status(&["merge.rs", "src/a.rs"], &[], &["merge.rs"]);
    let all = set(&["merge.rs", "src/a.rs"]);
    assert_eq!(formatter_paths(&status, &all, &all), paths(&["src/a.rs"]));
}

#[test]
fn a_path_listed_twice_is_passed_once_in_its_first_place() {
    let status = status(&["src/a.rs", "src/b.rs"], &["src/a.rs"], &[]);
    let all = set(&["src/a.rs", "src/b.rs"]);
    assert_eq!(formatter_paths(&status, &all, &all), paths(&["src/a.rs", "src/b.rs"]));
}

#[test]
fn an_empty_status_passes_nothing() {
    let all = set(&["src/a.rs"]);
    assert_eq!(formatter_paths(&StatusPaths::default(), &all, &all), Vec::<PathBuf>::new());
}
