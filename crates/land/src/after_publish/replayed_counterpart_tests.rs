//! replayed_counterpart: a match with another tree and parents is found; none listed is None; another author date,
//! another message (a different Mortar-Change-Id line) or another email is no match; of two matches the first
//! listed wins.

use super::replayed_counterpart;
use gitio::{CommitInfo, Oid, Person};

const SOURCE: &str = "1111111111111111111111111111111111111111";
const REPLAYED: &str = "2222222222222222222222222222222222222222";
const LATER: &str = "3333333333333333333333333333333333333333";
const TREE: &str = "4444444444444444444444444444444444444444";
const OTHER_TREE: &str = "5555555555555555555555555555555555555555";
const MESSAGE: &str = "published work\n\nMortar-Change-Id: Iaaaa\n";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn info(tree: &str, parents: &[&str], email: &str, date: &str, message: &str) -> CommitInfo {
    CommitInfo {
        tree: oid(tree),
        parents: parents.iter().map(|parent| oid(parent)).collect(),
        author: Person { name: "Zit".to_string(), email: email.to_string(), date: Some(date.to_string()) },
        message: message.to_string(),
    }
}

fn source() -> CommitInfo {
    info(TREE, &[LATER], "zit@example.invalid", "946684800 +0000", MESSAGE)
}

#[test]
fn a_match_with_another_tree_and_parents_is_found() {
    let replayed = info(OTHER_TREE, &[SOURCE], "zit@example.invalid", "946684800 +0000", MESSAGE);
    assert_eq!(replayed_counterpart(&source(), &[(oid(REPLAYED), replayed)]), Some(oid(REPLAYED)));
}

#[test]
fn none_listed_is_none() {
    assert_eq!(replayed_counterpart(&source(), &[]), None);
}

#[test]
fn the_same_message_with_another_author_date_is_no_match() {
    let other = info(OTHER_TREE, &[SOURCE], "zit@example.invalid", "946684801 +0000", MESSAGE);
    assert_eq!(replayed_counterpart(&source(), &[(oid(REPLAYED), other)]), None);
}

#[test]
fn the_same_author_with_another_message_is_no_match() {
    let message = "published work\n\nMortar-Change-Id: Ibbbb\n";
    let other = info(OTHER_TREE, &[SOURCE], "zit@example.invalid", "946684800 +0000", message);
    assert_eq!(replayed_counterpart(&source(), &[(oid(REPLAYED), other)]), None);
}

#[test]
fn the_same_message_and_author_name_with_another_email_is_no_match() {
    let other = info(OTHER_TREE, &[SOURCE], "other@example.invalid", "946684800 +0000", MESSAGE);
    assert_eq!(replayed_counterpart(&source(), &[(oid(REPLAYED), other)]), None);
}

#[test]
fn of_two_matches_the_first_listed_wins() {
    let first = info(OTHER_TREE, &[SOURCE], "zit@example.invalid", "946684800 +0000", MESSAGE);
    let second = info(TREE, &[REPLAYED], "zit@example.invalid", "946684800 +0000", MESSAGE);
    assert_eq!(replayed_counterpart(&source(), &[(oid(REPLAYED), first), (oid(LATER), second)]), Some(oid(REPLAYED)));
}
