//! would_merge_from: a clean merge is Yes, a conflicted one No with its files in order.

use super::{would_merge_from, WouldMerge};
use gitio::{MergeTreeOutcome, Oid};
use std::path::PathBuf;

/// The one helper building values only a constructor makes (Oid::parse).
fn tree() -> Oid {
    Oid::parse("4444444444444444444444444444444444444444").unwrap()
}

#[test]
fn clean_is_yes() {
    assert_eq!(would_merge_from(MergeTreeOutcome::Clean(tree())), WouldMerge::Yes);
}

#[test]
fn conflicted_is_no_with_its_files() {
    let outcome =
        MergeTreeOutcome::Conflicted { tree: tree(), files: vec![PathBuf::from("src/b.rs"), PathBuf::from("a.txt")] };
    assert_eq!(
        would_merge_from(outcome),
        WouldMerge::No { files: vec![PathBuf::from("src/b.rs"), PathBuf::from("a.txt")] }
    );
}
