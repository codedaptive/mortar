//! attempt_saved: a branch-only attempt is saved only when its trysnap tree equals its tip's tree (S-94).

use gitio::Oid;

use super::attempt_saved;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

#[test]
fn equal_trees_are_saved() {
    assert!(attempt_saved(Some(&oid('a')), &oid('a')));
}

#[test]
fn different_trees_are_not_saved() {
    assert!(!attempt_saved(Some(&oid('a')), &oid('b')));
}

#[test]
fn no_trysnap_ref_is_not_saved() {
    assert!(!attempt_saved(None, &oid('a')));
}
