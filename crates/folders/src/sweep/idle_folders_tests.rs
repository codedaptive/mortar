//! idle_folders: idle when nothing was found, or when the newest time is at least idle_days * 86400 seconds before
//! now; a time after now is never idle; input order is kept.

use std::path::PathBuf;

use super::idle_folders::idle_folders;
use super::FolderAge;

const NOW: u64 = 1_800_000_000;

fn age(path: &str, newest: Option<u64>) -> FolderAge {
    FolderAge { path: PathBuf::from(path), newest }
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

#[test]
fn exactly_at_the_boundary_is_idle() {
    // 3 days = 259200 seconds.
    assert_eq!(idle_folders(&[age("a", Some(1_799_740_800))], NOW, 3), paths(&["a"]));
}

#[test]
fn one_second_short_is_not_idle() {
    assert_eq!(idle_folders(&[age("a", Some(1_799_740_801))], NOW, 3), paths(&[]));
}

#[test]
fn a_time_after_now_is_not_idle() {
    assert_eq!(idle_folders(&[age("a", Some(1_800_000_001))], NOW, 0), paths(&[]));
}

#[test]
fn nothing_found_is_idle() {
    assert_eq!(idle_folders(&[age("a", None)], NOW, 30), paths(&["a"]));
}

#[test]
fn zero_days_picks_every_folder_at_or_before_now() {
    let ages = [age("a", Some(1_800_000_000)), age("b", Some(1_800_000_001)), age("c", Some(0))];
    assert_eq!(idle_folders(&ages, NOW, 0), paths(&["a", "c"]));
}

#[test]
fn a_huge_day_count_saturates_instead_of_overflowing() {
    let ages = [age("a", Some(0)), age("b", None)];
    assert_eq!(idle_folders(&ages, NOW, u64::MAX), paths(&["b"]));
}

#[test]
fn input_order_is_kept() {
    let ages = [
        age("z", None),
        age("busy", Some(1_799_999_000)),
        age("m", Some(1_000_000_000)),
        age("a", None),
    ];
    assert_eq!(idle_folders(&ages, NOW, 1), paths(&["z", "m", "a"]));
}
