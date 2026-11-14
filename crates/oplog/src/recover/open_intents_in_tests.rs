//! open_intents_in: the wrapping items whose intent no later done or failed record closes, in log order.

use super::open_intents_in;
use crate::store::{OpPhase, OpRecord, OpResult};

/// A log item that holds an op record beside something of its own.
#[derive(Debug, PartialEq)]
struct Tagged {
    record: OpRecord,
    tag: &'static str,
}

fn item(op_id: &str, phase: OpPhase, tag: &'static str) -> Tagged {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved { refs: Vec::new() }),
    };
    Tagged {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("key-{op_id}"),
            kind: "land".to_string(),
            actor: "simms".to_string(),
            reason: "squash lands".to_string(),
            at: 1763030400,
            phase,
            moves: vec![],
            result,
        },
        tag,
    }
}

fn tags(open: Vec<&Tagged>) -> Vec<&'static str> {
    open.into_iter().map(|t| t.tag).collect()
}

#[test]
fn an_empty_log_has_no_open_items() {
    let log: Vec<Tagged> = vec![];
    assert_eq!(tags(open_intents_in(&log, |t| &t.record)), Vec::<&str>::new());
}

#[test]
fn open_items_are_returned_in_log_order() {
    let log = vec![
        item("op-3", OpPhase::Intent, "a"),
        item("op-1", OpPhase::Intent, "b"),
        item("op-2", OpPhase::Intent, "c"),
        item("op-4", OpPhase::Intent, "d"),
    ];
    assert_eq!(tags(open_intents_in(&log, |t| &t.record)), vec!["a", "b", "c", "d"]);
}

#[test]
fn intents_closed_by_a_later_done_or_failed_are_left_out() {
    let log = vec![
        item("op-1", OpPhase::Intent, "a"),
        item("op-2", OpPhase::Intent, "b"),
        item("op-3", OpPhase::Intent, "c"),
        item("op-1", OpPhase::Done, "d"),
        item("op-2", OpPhase::Failed, "e"),
    ];
    assert_eq!(tags(open_intents_in(&log, |t| &t.record)), vec!["c"]);
}

#[test]
fn a_close_before_the_intent_does_not_close_it() {
    let log = vec![item("op-1", OpPhase::Done, "a"), item("op-1", OpPhase::Intent, "b")];
    assert_eq!(tags(open_intents_in(&log, |t| &t.record)), vec!["b"]);
}

#[test]
fn the_wrapping_item_is_returned_whole() {
    let log = vec![item("op-1", OpPhase::Intent, "kept")];
    assert_eq!(
        open_intents_in(&log, |t| &t.record),
        vec![&item("op-1", OpPhase::Intent, "kept")]
    );
}
