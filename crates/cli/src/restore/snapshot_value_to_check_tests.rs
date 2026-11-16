//! snapshot_value_to_check: a repeat checks against the done record's old value of the snapshot ref when it moved
//! that ref; a first attempt, or moves without that ref, check against the ref's present value (S-122(3)).

use gitio::Oid;
use oplog::RefMove;

use super::snapshot_value_to_check;

const SNAP: &str = "refs/mortar/snap/core/task";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn ref_move(refname: &str, old: Option<char>, new: char) -> RefMove {
    RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }
}

#[test]
fn a_repeat_with_the_snapshot_move_answers_that_move() {
    let moves = [ref_move(SNAP, Some('a'), 'b')];
    let found = snapshot_value_to_check(true, SNAP, &moves).expect("the snapshot move");
    assert_eq!(found.refname, "refs/mortar/snap/core/task");
    assert_eq!(found.old, Some(Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").expect("a valid oid")));
}

#[test]
fn a_repeat_with_a_snapshot_move_from_no_ref_answers_its_missing_old() {
    let moves = [ref_move(SNAP, None, 'b')];
    let found = snapshot_value_to_check(true, SNAP, &moves).expect("the snapshot move");
    assert_eq!(found.old, None);
}

#[test]
fn a_repeat_without_the_snapshot_move_answers_none() {
    let moves = [ref_move("refs/mortar/snap/core/other", Some('a'), 'b')];
    assert_eq!(snapshot_value_to_check(true, SNAP, &moves), None);
}

#[test]
fn a_first_attempt_with_the_snapshot_move_answers_none() {
    let moves = [ref_move(SNAP, Some('a'), 'b')];
    assert_eq!(snapshot_value_to_check(false, SNAP, &moves), None);
}

#[test]
fn a_repeat_with_several_moves_picks_the_matching_refname() {
    let moves = [
        ref_move("refs/heads/req/core/task", Some('1'), '2'),
        ref_move("refs/mortar/snap/core/other", Some('3'), '4'),
        ref_move(SNAP, Some('5'), '6'),
        ref_move("refs/mortar/trysnap/core/task/1", Some('7'), '8'),
    ];
    let found = snapshot_value_to_check(true, SNAP, &moves).expect("the snapshot move");
    assert_eq!(found.refname, "refs/mortar/snap/core/task");
    assert_eq!(found.old, Some(Oid::parse("5555555555555555555555555555555555555555").expect("a valid oid")));
    assert_eq!(found.new, Oid::parse("6666666666666666666666666666666666666666").expect("a valid oid"));
}

#[test]
fn a_request_ref_move_in_the_same_zone_is_not_the_snapshot_move() {
    let moves = [ref_move("refs/heads/req/core/task", Some('a'), 'b')];
    assert_eq!(snapshot_value_to_check(true, SNAP, &moves), None);
}
