//! index_tree reads a temporary copy of the folder's index: the folder's index file and index.lock are never touched,
//! and the copy is gone when index_tree returns, with a tree or with Unmerged.
use crate::fixture::{assert_no_temporary_index, committed, conflicted, stage, traced, tree_of};
use gitio::IndexTreeError;
use std::fs;
use std::path::PathBuf;

#[test]
fn a_held_index_lock_is_left_alone_and_the_tree_is_still_read() {
    let r = committed(&[("a.txt", b"base\n"), ("keep.txt", b"keep\n")]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n")]);
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n"), ("keep.txt", b"keep\n")]);
    let lock = r.path.join(".git/index.lock");
    fs::write(&lock, b"held by another process\n").unwrap();
    assert_eq!(r.git.index_tree(&r.path), Ok(expected));
    assert_eq!(fs::read(&lock).unwrap(), b"held by another process\n");
    fs::remove_file(lock).unwrap();
}

#[test]
fn the_folders_index_file_keeps_its_bytes_and_mtime() {
    let r = committed(&[("a.txt", b"base\n"), ("dir/b.txt", b"inner\n")]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n"), ("dir/new.txt", b"new\n")]);
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n"), ("dir/b.txt", b"inner\n"), ("dir/new.txt", b"new\n")]);
    let index = r.path.join(".git/index");
    let bytes = fs::read(&index).unwrap();
    let mtime = fs::metadata(&index).unwrap().modified().unwrap();
    assert_eq!(r.git.index_tree(&r.path), Ok(expected));
    assert_eq!(fs::read(&index).unwrap(), bytes, "the folder's index file was rewritten");
    assert_eq!(fs::metadata(&index).unwrap().modified().unwrap(), mtime);
}

#[test]
fn no_temporary_index_is_left_after_a_tree() {
    let r = committed(&[("a.txt", b"base\n")]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n")]);
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n")]);
    let (git, trace) = traced(&r);
    assert_eq!(git.index_tree(&r.path), Ok(expected));
    assert_no_temporary_index(&trace);
}

#[test]
fn no_temporary_index_is_left_after_unmerged() {
    let r = conflicted();
    let (git, trace) = traced(&r);
    let paths = ["a.txt", "dir/sp ace.txt", "z.txt"].map(PathBuf::from).to_vec();
    assert_eq!(git.index_tree(&r.path), Err(IndexTreeError::Unmerged { paths }));
    assert_no_temporary_index(&trace);
}
