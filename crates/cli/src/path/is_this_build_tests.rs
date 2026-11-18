//! is_this_build: true for equal canonical paths, or different ones whose canonical candidate carries the marker;
//! false without the marker or when either path will not canonicalize; the marker is never asked when the canonical
//! paths are equal. `canonical` is a fake over a map; `has_marker` a fake over a list of canonical paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::is_this_build;

const THIS: &str = "/b/target/debug/mortar";

fn canonical_from(pairs: &[(&str, &str)]) -> impl Fn(&Path) -> Option<PathBuf> {
    let map: HashMap<PathBuf, PathBuf> =
        pairs.iter().map(|(from, to)| (PathBuf::from(from), PathBuf::from(to))).collect();
    move |path| map.get(path).cloned()
}

fn marked_by(marked: &[&str]) -> impl Fn(&Path) -> bool {
    let marked: Vec<PathBuf> = marked.iter().map(PathBuf::from).collect();
    move |path| marked.iter().any(|m| m == path)
}

fn refuse(path: &Path) -> bool {
    panic!("has_marker asked of {}", path.display())
}

#[test]
fn equal_canonical_paths_are_this_build_without_asking_the_marker() {
    let pairs = [(THIS, THIS), ("/usr/local/bin/mr", THIS)];
    assert!(is_this_build(Path::new("/usr/local/bin/mr"), Path::new(THIS), &canonical_from(&pairs), &refuse));
}

#[test]
fn the_same_path_given_twice_is_this_build_without_asking_the_marker() {
    let pairs = [(THIS, THIS)];
    assert!(is_this_build(Path::new(THIS), Path::new(THIS), &canonical_from(&pairs), &refuse));
}

#[test]
fn a_different_path_carrying_the_marker_is_this_build() {
    let pairs = [(THIS, THIS), ("/home/u/.cargo/bin/mr", "/home/u/.cargo/bin/mr")];
    let marked = marked_by(&["/home/u/.cargo/bin/mr"]);
    assert!(is_this_build(Path::new("/home/u/.cargo/bin/mr"), Path::new(THIS), &canonical_from(&pairs), &marked));
}

#[test]
fn the_marker_is_asked_of_the_canonical_candidate() {
    let pairs = [(THIS, THIS), ("/home/u/bin/mr", "/opt/mortar/bin/mortar")];
    let canonical = canonical_from(&pairs);
    let candidate = Path::new("/home/u/bin/mr");
    assert!(is_this_build(candidate, Path::new(THIS), &canonical, &marked_by(&["/opt/mortar/bin/mortar"])));
    assert!(!is_this_build(candidate, Path::new(THIS), &canonical, &marked_by(&["/home/u/bin/mr"])));
}

#[test]
fn a_different_path_without_the_marker_is_not_this_build() {
    let pairs = [(THIS, THIS), ("/usr/local/bin/mr", "/usr/local/bin/mr")];
    assert!(!is_this_build(Path::new("/usr/local/bin/mr"), Path::new(THIS), &canonical_from(&pairs), &marked_by(&[])));
}

#[test]
fn a_candidate_that_will_not_canonicalize_is_not_this_build() {
    let pairs = [(THIS, THIS)];
    let marked = marked_by(&["/b/target/debug/mr"]);
    assert!(!is_this_build(Path::new("/b/target/debug/mr"), Path::new(THIS), &canonical_from(&pairs), &marked));
}

#[test]
fn this_binary_that_will_not_canonicalize_is_not_this_build() {
    let pairs = [("/b/target/debug/mr", "/b/target/debug/mr")];
    let marked = marked_by(&["/b/target/debug/mr"]);
    assert!(!is_this_build(Path::new("/b/target/debug/mr"), Path::new(THIS), &canonical_from(&pairs), &marked));
}

#[test]
fn neither_path_canonicalizing_is_not_this_build() {
    assert!(!is_this_build(Path::new(THIS), Path::new(THIS), &canonical_from(&[]), &refuse));
}
