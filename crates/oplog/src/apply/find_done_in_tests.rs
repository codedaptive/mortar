//! find_done_in: over items wrapping a record, the first item whose record is done with the key; intents,
//! failures and other keys do not count.

use super::find_done_in;
use crate::store::{OpPhase, OpRecord, OpResult};

#[derive(Debug, PartialEq)]
struct Tagged {
    record: OpRecord,
    tag: &'static str,
}

fn tagged(tag: &'static str, key: &str, phase: OpPhase) -> Tagged {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved { refs: Vec::new() }),
    };
    Tagged {
        record: OpRecord {
            op_id: "op-1".to_string(),
            key: key.to_string(),
            kind: "land".to_string(),
            actor: "simms".to_string(),
            reason: "squash lands".to_string(),
            at: 1,
            phase,
            moves: Vec::new(),
            result,
        },
        tag,
    }
}

fn find(items: &[Tagged], key: &str) -> Option<&'static str> {
    find_done_in(items, key, |t| &t.record).map(|t| t.tag)
}

#[test]
fn returns_the_wrapping_item_of_the_done_record() {
    let items = vec![
        tagged("intent", "k", OpPhase::Intent),
        tagged("done", "k", OpPhase::Done),
    ];
    assert_eq!(
        find_done_in(&items, "k", |t| &t.record),
        Some(&tagged("done", "k", OpPhase::Done))
    );
}

#[test]
fn the_first_of_two_done_items_is_returned() {
    let items = vec![
        tagged("first", "k", OpPhase::Done),
        tagged("second", "k", OpPhase::Done),
    ];
    assert_eq!(find(&items, "k"), Some("first"));
}

#[test]
fn intent_and_failed_items_with_the_key_are_skipped() {
    let items = vec![
        tagged("intent", "k", OpPhase::Intent),
        tagged("failed", "k", OpPhase::Failed),
    ];
    assert_eq!(find(&items, "k"), None);
}

#[test]
fn a_done_item_with_another_key_is_skipped() {
    let items = vec![
        tagged("other", "other", OpPhase::Done),
        tagged("longer", "k-longer", OpPhase::Done),
        tagged("match", "k", OpPhase::Done),
    ];
    assert_eq!(find(&items, "k"), Some("match"));
}

#[test]
fn an_empty_slice_has_no_done_item() {
    assert_eq!(find(&[], "k"), None);
}
