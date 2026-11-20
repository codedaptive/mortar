//! accumulate_paths: earlier paths first, then the round's new ones in order, never a path twice.

use crate::accumulate_paths::accumulate_paths;
use std::path::PathBuf;

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

#[test]
fn no_earlier_paths_gives_the_round_in_order() {
    assert_eq!(accumulate_paths(&[], &paths(&["src/b.rs", "a.txt"])), paths(&["src/b.rs", "a.txt"]));
}

#[test]
fn disjoint_rounds_keep_earlier_first_then_the_round() {
    assert_eq!(
        accumulate_paths(&paths(&["z.txt", "src/a.rs"]), &paths(&["b.txt", "lib/c.rs"])),
        paths(&["z.txt", "src/a.rs", "b.txt", "lib/c.rs"])
    );
}

#[test]
fn a_path_seen_in_an_earlier_round_is_not_added_again() {
    assert_eq!(
        accumulate_paths(&paths(&["src/a.rs", "b.txt"]), &paths(&["b.txt", "c.md", "src/a.rs"])),
        paths(&["src/a.rs", "b.txt", "c.md"])
    );
}

#[test]
fn a_repeat_within_the_round_is_listed_once() {
    assert_eq!(
        accumulate_paths(&paths(&["a.txt"]), &paths(&["d.txt", "d.txt", "e.txt", "d.txt"])),
        paths(&["a.txt", "d.txt", "e.txt"])
    );
}

#[test]
fn an_empty_round_leaves_the_earlier_paths() {
    assert_eq!(accumulate_paths(&paths(&["a.txt", "b.txt"]), &[]), paths(&["a.txt", "b.txt"]));
}
