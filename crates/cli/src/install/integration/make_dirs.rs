//! Directory creation preserves the list made before a later filesystem error.

use std::fs;

use super::super::make_dirs;
use super::support::Tree;

#[test]
fn make_dirs_returns_the_folders_it_made_before_a_later_failure() {
    let tree = Tree::new();
    let a = tree.root.join("a");
    let b = a.join("b");
    let long = b.join("x".repeat(256));
    let (made, result) = make_dirs(&long);
    assert_eq!(made, vec![a, b.clone()]);
    assert!(result.is_err());
    assert!(fs::symlink_metadata(&b).unwrap().file_type().is_dir());
    assert!(fs::symlink_metadata(&long).is_err());
}
