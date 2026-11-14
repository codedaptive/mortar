//! Which index is read: the folder's own, for a linked worktree and for a subfolder of a worktree.
use crate::fixture::{committed, raw, stage, tree_of};

#[test]
fn a_linked_worktree_reads_its_own_index_and_the_main_reads_its_own() {
    let r = committed(&[("a.txt", b"base\n"), ("keep.txt", b"keep\n")]);
    let linked = r.path.join("linked folder");
    raw(&r.git, &r.path, &["worktree", "add", "--quiet", "--detach", linked.to_str().unwrap(), "HEAD"]);
    stage(&r.git, &linked, &[("a.txt", b"linked\n")]);
    stage(&r.git, &r.path, &[("a.txt", b"main\n")]);
    let linked_tree = tree_of(&r.git, &[("a.txt", b"linked\n"), ("keep.txt", b"keep\n")]);
    let main_tree = tree_of(&r.git, &[("a.txt", b"main\n"), ("keep.txt", b"keep\n")]);
    assert_eq!(r.git.index_tree(&linked).unwrap(), linked_tree);
    assert_eq!(r.git.index_tree(&r.path).unwrap(), main_tree);
}

#[test]
fn a_subfolder_of_a_clean_worktree_gives_the_same_tree_as_the_top() {
    let r = committed(&[("a.txt", b"base\n"), ("dir/b.txt", b"inner\n")]);
    stage(&r.git, &r.path, &[("a.txt", b"staged\n")]);
    let expected = tree_of(&r.git, &[("a.txt", b"staged\n"), ("dir/b.txt", b"inner\n")]);
    assert_eq!(r.git.index_tree(&r.path.join("dir")).unwrap(), expected);
    assert_eq!(r.git.index_tree(&r.path).unwrap(), expected);
}
