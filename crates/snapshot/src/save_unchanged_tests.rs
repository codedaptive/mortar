use crate::save_unchanged::save_unchanged;
use gitio::Oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn equal_trees_are_unchanged() {
    assert!(save_unchanged(
        &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    ));
}

#[test]
fn different_trees_are_changed() {
    assert!(!save_unchanged(
        &oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        &oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    ));
}
