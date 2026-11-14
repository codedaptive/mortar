//! on_first_parent_line: on the line when base is tip or the oldest commit's first parent; off it otherwise.

use gitio::Oid;

use super::on_first_parent_line;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).unwrap()
}

#[test]
fn base_equal_to_tip_is_on_the_line() {
    assert!(on_first_parent_line(&oid('a'), &oid('a'), &[], None));
}

#[test]
fn base_as_the_oldest_commits_first_parent_is_on_the_line() {
    let line = [oid('b'), oid('c'), oid('d')];
    assert!(on_first_parent_line(&oid('a'), &oid('d'), &line, Some(&oid('a'))));
}

#[test]
fn base_reached_only_as_a_merges_second_parent_is_off_the_line() {
    // line[0] is the merge: base is its second parent, so the walk passes it and reaches the merge's first
    // parent e instead.
    let line = [oid('b'), oid('c')];
    assert!(!on_first_parent_line(&oid('a'), &oid('c'), &line, Some(&oid('e'))));
}

#[test]
fn base_that_is_no_ancestor_is_off_the_line() {
    // The walk runs down to a root commit, whose first parent is none.
    let line = [oid('b'), oid('c')];
    assert!(!on_first_parent_line(&oid('a'), &oid('c'), &line, None));
}

#[test]
fn tip_already_reached_from_base_is_off_the_line() {
    assert!(!on_first_parent_line(&oid('a'), &oid('b'), &[], None));
}
