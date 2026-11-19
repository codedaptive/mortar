//! find_zone_old: the commit a zone folder stands on: the first value, in the order given, whose tree is the
//! folder's index tree; an index at the tip's tree reads no tree at all, and an index matching no value is NoMatch.

use std::collections::HashMap;

use gitio::Oid;

use super::find_zone_old;
use crate::after_publish::ZoneOld;

const INDEX_TREE: &str = "7f2e4d6c8b0a1f3e5d7c9b2a4f6e8d0c1b3a5f79";
const TIP_TREE: &str = "b1d3f5a7c9e2b4d6f8a0c3e5b7d9f1a2c4e6b8d0";
const OTHER_TREE: &str = "0d9c8b7a6f5e4d3c2b1a0f9e8d7c6b5a4f3e2d1c";
const TIP: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const A: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const B: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const C: &str = "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// Runs find_zone_old with a tree reader that answers from `trees` (None for a value not listed) and records each
/// value it was asked for, in order.
fn find(index_tree: &str, tip_tree: &str, values: &[&str], trees: &[(&str, &str)]) -> (ZoneOld, Vec<Oid>) {
    let trees: HashMap<Oid, Oid> = trees.iter().map(|(commit, tree)| (oid(commit), oid(tree))).collect();
    let values: Vec<Oid> = values.iter().map(|hex| oid(hex)).collect();
    let mut asked = Vec::new();
    let answer = find_zone_old(&oid(index_tree), &oid(tip_tree), &values, |value: &Oid| {
        asked.push(value.clone());
        trees.get(value).cloned()
    });
    (answer, asked)
}

#[test]
fn an_index_at_the_tips_tree_is_at_tip_and_reads_no_tree() {
    let (answer, asked) =
        find(TIP_TREE, TIP_TREE, &[TIP, A, B], &[(TIP, TIP_TREE), (A, INDEX_TREE), (B, OTHER_TREE)]);
    assert_eq!(answer, ZoneOld::AtTip);
    assert_eq!(asked, Vec::<Oid>::new());
}

#[test]
fn the_first_value_whose_tree_is_the_index_tree_is_taken_and_no_further_value_is_read() {
    let (answer, asked) = find(
        INDEX_TREE,
        TIP_TREE,
        &[TIP, A, B, C],
        &[(TIP, TIP_TREE), (A, INDEX_TREE), (B, INDEX_TREE), (C, OTHER_TREE)],
    );
    assert_eq!(answer, ZoneOld::From(oid(A)));
    assert_eq!(asked, vec![oid(TIP), oid(A)]);
}

#[test]
fn a_later_value_is_taken_when_the_earlier_ones_differ() {
    let (answer, asked) =
        find(INDEX_TREE, TIP_TREE, &[TIP, A, B], &[(TIP, TIP_TREE), (A, OTHER_TREE), (B, INDEX_TREE)]);
    assert_eq!(answer, ZoneOld::From(oid(B)));
    assert_eq!(asked, vec![oid(TIP), oid(A), oid(B)]);
}

#[test]
fn a_value_whose_tree_cannot_be_read_is_passed_over() {
    let (answer, asked) = find(INDEX_TREE, TIP_TREE, &[TIP, A, B], &[(TIP, TIP_TREE), (B, INDEX_TREE)]);
    assert_eq!(answer, ZoneOld::From(oid(B)));
    assert_eq!(asked, vec![oid(TIP), oid(A), oid(B)]);
}

#[test]
fn no_value_with_the_index_tree_is_no_match_after_reading_every_value() {
    let (answer, asked) =
        find(INDEX_TREE, TIP_TREE, &[TIP, A, B], &[(TIP, TIP_TREE), (A, OTHER_TREE), (B, OTHER_TREE)]);
    assert_eq!(answer, ZoneOld::NoMatch);
    assert_eq!(asked, vec![oid(TIP), oid(A), oid(B)]);
}

#[test]
fn values_none_of_whose_trees_can_be_read_are_no_match() {
    let (answer, asked) = find(INDEX_TREE, TIP_TREE, &[A, B], &[]);
    assert_eq!(answer, ZoneOld::NoMatch);
    assert_eq!(asked, vec![oid(A), oid(B)]);
}

#[test]
fn no_values_and_an_index_off_the_tip_is_no_match() {
    let (answer, asked) = find(INDEX_TREE, TIP_TREE, &[], &[]);
    assert_eq!(answer, ZoneOld::NoMatch);
    assert_eq!(asked, Vec::<Oid>::new());
}

/// The staged-edit loss case as data. The zone branch went A -> B (tip B, tree TIP_TREE; A's tree OTHER_TREE); the
/// folder's index holds tree INDEX_TREE: A's files plus an edit staged at a path B changes. INDEX_TREE matches
/// neither value, so the folder is refused and the staged edit kept. A move from INDEX_TREE itself would let git's
/// two-tree move replace the staged edit with B's copy.
#[test]
fn a_staged_edit_matches_no_value_the_zone_held_so_the_folder_is_not_moved() {
    let (answer, asked) = find(INDEX_TREE, TIP_TREE, &[B, A], &[(B, TIP_TREE), (A, OTHER_TREE)]);
    assert_eq!(answer, ZoneOld::NoMatch);
    assert_eq!(asked, vec![oid(B), oid(A)]);
}
