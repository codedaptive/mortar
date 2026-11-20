//! same_build: equal canonical paths, or a candidate whose bytes carry the Mortar marker; a name beside this
//! binary counts for nothing. Pure: none of these paths exist.

use std::path::Path;

use super::same_build;

#[test]
fn equal_paths_are_the_same_build() {
    assert!(same_build(Path::new("/b/target/debug/mortar"), Path::new("/b/target/debug/mortar"), false));
}

#[test]
fn equal_paths_with_any_name_are_the_same_build() {
    assert!(same_build(Path::new("/opt/tools/mortar-dev"), Path::new("/opt/tools/mortar-dev"), false));
}

#[test]
fn a_copied_mortar_elsewhere_carrying_the_marker_is_the_same_build() {
    assert!(same_build(Path::new("/home/u/.cargo/bin/mr"), Path::new("/b/target/debug/mortar"), true));
}

#[test]
fn another_path_without_the_marker_is_not_the_same_build() {
    assert!(!same_build(Path::new("/usr/local/bin/mr"), Path::new("/b/target/debug/mortar"), false));
}

#[test]
fn an_mr_beside_this_binary_without_the_marker_is_not_the_same_build() {
    assert!(!same_build(Path::new("/b/target/debug/mr"), Path::new("/b/target/debug/mortar"), false));
}

#[test]
fn a_mortar_beside_this_binary_without_the_marker_is_not_the_same_build() {
    assert!(!same_build(Path::new("/b/target/debug/mortar"), Path::new("/b/target/debug/mr"), false));
}

#[test]
fn an_mr_beside_this_binary_carrying_the_marker_is_the_same_build() {
    assert!(same_build(Path::new("/b/target/debug/mr"), Path::new("/b/target/debug/mortar"), true));
}
