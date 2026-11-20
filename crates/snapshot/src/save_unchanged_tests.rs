use crate::save_unchanged::save_unchanged;
use gitio::Oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn equal_trees_set_the_index_to_heads_tree() {
    assert_eq!(
        save_unchanged(
            &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            false
        ),
        Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
    );
}

#[test]
fn different_trees_have_a_commit_to_write() {
    assert_eq!(
        save_unchanged(
            &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            &oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            false
        ),
        None
    );
}

#[test]
fn equal_trees_with_head_moved_take_the_commit_path() {
    assert_eq!(
        save_unchanged(
            &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            true
        ),
        None
    );
}

#[test]
fn different_trees_with_head_moved_take_the_commit_path() {
    assert_eq!(
        save_unchanged(
            &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            &oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            true
        ),
        None
    );
}
