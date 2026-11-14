use crate::absent_paths::absent_paths;
use crate::Oid;
use std::path::PathBuf;

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

fn entry(mode: &str, hex: &str) -> Option<(String, Oid)> {
    Some((mode.to_string(), Oid::parse(hex).unwrap()))
}

#[test]
fn mixed_keeps_the_absent_paths_in_order_and_duplicates() {
    let entries = [
        None,
        entry("100644", "1111111111111111111111111111111111111111"),
        None,
        None,
    ];
    assert_eq!(absent_paths(&paths(&["z", "kept", "a", "z"]), &entries), paths(&["z", "a", "z"]));
}

#[test]
fn none_absent_is_empty() {
    let entries = [
        entry("100644", "1111111111111111111111111111111111111111"),
        entry("160000", "2222222222222222222222222222222222222222"),
    ];
    assert_eq!(absent_paths(&paths(&["a", "sub"]), &entries), Vec::<PathBuf>::new());
}

#[test]
fn all_absent_is_every_path() {
    assert_eq!(absent_paths(&paths(&["a", "b/c"]), &[None, None]), paths(&["a", "b/c"]));
}

#[test]
fn no_paths_is_empty() {
    assert_eq!(absent_paths(&[], &[]), Vec::<PathBuf>::new());
}
