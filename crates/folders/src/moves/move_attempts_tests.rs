//! move_attempts: the plain move alone without an index tree; the plain move then set-index-and-move with one.

use gitio::Oid;

use super::move_attempts::move_attempts;
use super::MoveAttempt;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn no_index_tree_is_one_plain_move_from_old() {
    let old = oid("0123456789abcdef0123456789abcdef01234567");
    assert_eq!(
        move_attempts(&old, None),
        vec![MoveAttempt { set_index: None, from: &oid("0123456789abcdef0123456789abcdef01234567") }]
    );
}

#[test]
fn index_tree_is_the_plain_move_then_set_index_and_move_from_the_tree() {
    let old = oid("0123456789abcdef0123456789abcdef01234567");
    let tree = oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let want_old = oid("0123456789abcdef0123456789abcdef01234567");
    let want_tree = oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(
        move_attempts(&old, Some(&tree)),
        vec![
            MoveAttempt { set_index: None, from: &want_old },
            MoveAttempt { set_index: Some(&want_tree), from: &want_tree },
        ]
    );
}

#[test]
fn index_tree_equal_to_old_still_gives_both_tries() {
    let old = oid("0123456789abcdef0123456789abcdef01234567");
    let tree = oid("0123456789abcdef0123456789abcdef01234567");
    let want = oid("0123456789abcdef0123456789abcdef01234567");
    assert_eq!(
        move_attempts(&old, Some(&tree)),
        vec![MoveAttempt { set_index: None, from: &want }, MoveAttempt { set_index: Some(&want), from: &want }]
    );
}
