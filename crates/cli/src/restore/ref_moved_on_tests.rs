//! ref_moved_on: a repeat leaves a folder whose ref no longer holds the record's new value (or is missing); a first
//! attempt never does.

use gitio::Oid;

use super::ref_moved_on;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

#[test]
fn a_repeat_with_the_ref_at_the_new_value_has_not_moved_on() {
    assert!(!ref_moved_on(true, Some(&oid('a')), &oid('a')));
}

#[test]
fn a_repeat_with_the_ref_at_another_value_has_moved_on() {
    assert!(ref_moved_on(true, Some(&oid('b')), &oid('a')));
}

#[test]
fn a_repeat_with_the_ref_missing_has_moved_on() {
    assert!(ref_moved_on(true, None, &oid('a')));
}

#[test]
fn a_first_attempt_with_the_ref_at_another_value_has_not_moved_on() {
    assert!(!ref_moved_on(false, Some(&oid('b')), &oid('a')));
}

#[test]
fn a_first_attempt_with_the_ref_at_the_new_value_has_not_moved_on() {
    assert!(!ref_moved_on(false, Some(&oid('a')), &oid('a')));
}
