//! Index forms the copy must read as the folder's own index does: none yet (a fresh repository), a split index
//! (core.splitIndex, its shared part in the folder's git dir) and a sparse index (a cone with a collapsed directory).
use crate::fixture::{committed, raw, repo, stage, tree_of};
use gitio::Oid;
use std::fs;

/// git's empty tree, the tree of an index with no entries.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

#[test]
fn a_fresh_repository_with_no_index_gives_the_empty_tree_and_no_index_file() {
    let r = repo();
    let index = r.path.join(".git/index");
    assert!(!index.exists(), "the fixture must have no index yet");
    let tree = r.git.index_tree(&r.path);
    assert!(!index.exists(), "index_tree made an index file in the folder's git dir");
    assert_eq!(tree, Ok(Oid::parse(EMPTY_TREE).unwrap()));
}

#[test]
fn a_split_index_with_a_staged_change_gives_the_staged_tree() {
    let r = committed(&[("a.txt", b"base\n"), ("keep.txt", b"keep\n")]);
    raw(&r.git, &r.path, &["config", "core.splitIndex", "true"]);
    raw(&r.git, &r.path, &["update-index", "--split-index"]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n")]);
    let shared = fs::read_dir(r.path.join(".git")).unwrap()
        .any(|item| item.unwrap().file_name().to_string_lossy().starts_with("sharedindex."));
    assert!(shared, "the fixture must hold a split index");
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n"), ("keep.txt", b"keep\n")]);
    assert_eq!(r.git.index_tree(&r.path), Ok(expected));
}

#[test]
fn a_sparse_index_with_a_staged_change_in_the_cone_gives_the_staged_tree() {
    let r = committed(&[("in/a.txt", b"base\n"), ("out/b.txt", b"outside\n"), ("top.txt", b"top\n")]);
    raw(&r.git, &r.path, &["sparse-checkout", "set", "--cone", "--sparse-index", "in"]);
    let listed = raw(&r.git, &r.path, &["ls-files", "--sparse"]);
    assert!(listed.split(|&b| b == b'\n').any(|line| line == b"out/"), "the fixture must collapse out/");
    stage(&r.git, &r.path, &[("in/a.txt", b"staged\n")]);
    let expected = tree_of(&r.git, &[("in/a.txt", b"staged\n"), ("out/b.txt", b"outside\n"), ("top.txt", b"top\n")]);
    assert_eq!(r.git.index_tree(&r.path), Ok(expected));
}
