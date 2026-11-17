//! pick_squash_ref: the one squash ref carrying the change ID, none, or several in input order; entries whose
//! change ID could not be read never match.

use gitio::Oid;
use oplog::ChangeId;

use super::pick_squash_ref;
use crate::landing::SquashPick;

const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const OTHER: &str = "zzzzyyyyxxxxwwwwvvvvuuuuttttssss";
const COMMIT_A: &str = "1111111111111111111111111111111111111111";
const COMMIT_B: &str = "2222222222222222222222222222222222222222";
const COMMIT_C: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn change(letters: &str) -> ChangeId {
    ChangeId::parse(letters).unwrap()
}

fn entry(zone: &str, commit: &str, carried: Option<&str>) -> (String, Oid, Option<ChangeId>) {
    (format!("refs/mortar/squash/{zone}"), oid(commit), carried.map(change))
}

#[test]
fn no_squash_refs_is_missing() {
    assert_eq!(pick_squash_ref(&change(CHANGE), &[]), SquashPick::Missing);
}

#[test]
fn the_one_ref_carrying_the_change_is_picked_with_its_commit() {
    let squashes = [entry("core", COMMIT_A, Some(OTHER)), entry("web", COMMIT_B, Some(CHANGE))];
    assert_eq!(
        pick_squash_ref(&change(CHANGE), &squashes),
        SquashPick::One { refname: "refs/mortar/squash/web".to_string(), commit: oid(COMMIT_B) }
    );
}

#[test]
fn refs_carrying_only_other_changes_are_missing() {
    let squashes = [entry("core", COMMIT_A, Some(OTHER)), entry("web", COMMIT_B, Some(OTHER))];
    assert_eq!(pick_squash_ref(&change(CHANGE), &squashes), SquashPick::Missing);
}

#[test]
fn refs_that_could_not_be_read_never_match() {
    let squashes = [entry("core", COMMIT_A, None), entry("web", COMMIT_B, None)];
    assert_eq!(pick_squash_ref(&change(CHANGE), &squashes), SquashPick::Missing);
}

#[test]
fn an_unreadable_ref_beside_the_carrying_one_leaves_it_the_one() {
    let squashes = [entry("core", COMMIT_A, None), entry("web", COMMIT_B, Some(CHANGE)), entry("db", COMMIT_C, None)];
    assert_eq!(
        pick_squash_ref(&change(CHANGE), &squashes),
        SquashPick::One { refname: "refs/mortar/squash/web".to_string(), commit: oid(COMMIT_B) }
    );
}

#[test]
fn several_refs_carrying_the_change_are_named_in_input_order() {
    let squashes = [
        entry("web", COMMIT_B, Some(CHANGE)),
        entry("core", COMMIT_A, Some(OTHER)),
        entry("db", COMMIT_C, Some(CHANGE)),
        entry("api", COMMIT_A, Some(CHANGE)),
    ];
    assert_eq!(
        pick_squash_ref(&change(CHANGE), &squashes),
        SquashPick::Several {
            refnames: vec![
                "refs/mortar/squash/web".to_string(),
                "refs/mortar/squash/db".to_string(),
                "refs/mortar/squash/api".to_string(),
            ]
        }
    );
}
