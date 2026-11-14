//! same_squash: the repeat decision; same only for the same tree, the one same parent and the exact message.

use super::same_squash;
use gitio::{CommitInfo, Oid, Person};

const TREE: &str = "1111111111111111111111111111111111111111";
const PARENT: &str = "2222222222222222222222222222222222222222";
const OTHER: &str = "3333333333333333333333333333333333333333";
const MESSAGE: &str = "land: 1 requests\n\nr1: Add a\n\nMortar-Zone: land\n";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn info(tree: &str, parents: &[&str], message: &str) -> CommitInfo {
    CommitInfo {
        tree: oid(tree),
        parents: parents.iter().map(|p| oid(p)).collect(),
        author: Person { name: "A".to_string(), email: "a@x".to_string(), date: Some("1 +0000".to_string()) },
        message: message.to_string(),
    }
}

#[test]
fn same_tree_parent_and_message_is_the_same() {
    assert!(same_squash(&info(TREE, &[PARENT], MESSAGE), &oid(TREE), &oid(PARENT), MESSAGE));
}

#[test]
fn a_different_tree_is_not_the_same() {
    assert!(!same_squash(&info(OTHER, &[PARENT], MESSAGE), &oid(TREE), &oid(PARENT), MESSAGE));
}

#[test]
fn a_different_parent_is_not_the_same() {
    assert!(!same_squash(&info(TREE, &[OTHER], MESSAGE), &oid(TREE), &oid(PARENT), MESSAGE));
}

#[test]
fn two_parents_are_not_the_same_even_when_the_first_matches() {
    assert!(!same_squash(&info(TREE, &[PARENT, OTHER], MESSAGE), &oid(TREE), &oid(PARENT), MESSAGE));
}

#[test]
fn no_parent_is_not_the_same() {
    assert!(!same_squash(&info(TREE, &[], MESSAGE), &oid(TREE), &oid(PARENT), MESSAGE));
}

#[test]
fn a_message_one_char_different_is_not_the_same() {
    let prior = "land: 1 requests\n\nr1: Add b\n\nMortar-Zone: land\n";
    assert!(!same_squash(&info(TREE, &[PARENT], prior), &oid(TREE), &oid(PARENT), MESSAGE));
}

#[test]
fn a_message_differing_only_by_a_final_newline_is_not_the_same() {
    let prior = "land: 1 requests\n\nr1: Add a\n\nMortar-Zone: land";
    assert!(!same_squash(&info(TREE, &[PARENT], prior), &oid(TREE), &oid(PARENT), MESSAGE));
}
