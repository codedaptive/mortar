//! What index_tree reads: the index only, its staged changes and not unstaged ones, the same id twice, no ref moved.
use crate::fixture::{committed, head_tree, raw, stage, tree_of};
use std::fs;

#[test]
fn a_clean_index_gives_heads_tree() {
    let r = committed(&[("a.txt", b"base\n"), ("dir/b.txt", b"inner\n")]);
    let expected = head_tree(&r.git, &r.path);
    assert_eq!(r.git.index_tree(&r.path).unwrap(), expected);
}

#[test]
fn a_staged_change_gives_the_staged_tree_and_moves_no_ref() {
    let r = committed(&[("a.txt", b"base\n"), ("keep.txt", b"keep\n")]);
    raw(&r.git, &r.path, &["branch", "side"]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n"), ("new file.txt", b"new\n")]);
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n"), ("keep.txt", b"keep\n"), ("new file.txt", b"new\n")]);
    assert_ne!(expected, head_tree(&r.git, &r.path), "the fixture must stage a real change");
    let head_file = fs::read(r.path.join(".git/HEAD")).unwrap();
    let head = raw(&r.git, &r.path, &["rev-parse", "HEAD"]);
    let refs = raw(&r.git, &r.path, &["for-each-ref"]);
    let reflog = fs::read(r.path.join(".git/logs/HEAD")).unwrap();
    assert_eq!(r.git.index_tree(&r.path).unwrap(), expected);
    assert_eq!(fs::read(r.path.join(".git/HEAD")).unwrap(), head_file);
    assert_eq!(raw(&r.git, &r.path, &["rev-parse", "HEAD"]), head);
    assert_eq!(raw(&r.git, &r.path, &["for-each-ref"]), refs);
    assert_eq!(fs::read(r.path.join(".git/logs/HEAD")).unwrap(), reflog);
}

#[test]
fn an_unstaged_edit_is_not_in_the_tree() {
    let r = committed(&[("a.txt", b"base\n"), ("keep.txt", b"keep\n")]);
    fs::write(r.path.join("a.txt"), b"unstaged edit\n").unwrap();
    fs::write(r.path.join("untracked.txt"), b"untracked\n").unwrap();
    let expected = head_tree(&r.git, &r.path);
    assert_eq!(r.git.index_tree(&r.path).unwrap(), expected);
}

#[test]
fn a_second_call_gives_the_same_id() {
    let r = committed(&[("a.txt", b"base\n")]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n")]);
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n")]);
    let first = r.git.index_tree(&r.path).unwrap();
    let second = r.git.index_tree(&r.path).unwrap();
    assert_eq!(first, expected);
    assert_eq!(second, expected);
}
