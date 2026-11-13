//! open_intents: intents with no later done or failed record, in log order.

use super::open_intents;
use crate::store::{OpPhase, OpRecord, OpResult};

fn rec(op_id: &str, phase: OpPhase) -> OpRecord {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved),
    };
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase,
        moves: vec![],
        result,
    }
}

fn ids(open: Vec<&OpRecord>) -> Vec<&str> {
    open.into_iter().map(|r| r.op_id.as_str()).collect()
}

#[test]
fn an_empty_log_has_no_open_intents() {
    assert_eq!(ids(open_intents(&[])), Vec::<&str>::new());
}

#[test]
fn intents_closed_by_done_or_failed_are_ignored() {
    let log = vec![
        rec("op-1", OpPhase::Intent),
        rec("op-2", OpPhase::Intent),
        rec("op-1", OpPhase::Done),
        rec("op-2", OpPhase::Failed),
    ];
    assert_eq!(ids(open_intents(&log)), Vec::<&str>::new());
}

#[test]
fn open_intents_are_kept_in_log_order() {
    let log = vec![
        rec("op-3", OpPhase::Intent),
        rec("op-1", OpPhase::Intent),
        rec("op-2", OpPhase::Intent),
        rec("op-1", OpPhase::Done),
        rec("op-4", OpPhase::Intent),
    ];
    assert_eq!(ids(open_intents(&log)), vec!["op-3", "op-2", "op-4"]);
}

#[test]
fn the_open_intent_is_returned_whole() {
    let log = vec![rec("op-1", OpPhase::Intent)];
    assert_eq!(open_intents(&log), vec![&rec("op-1", OpPhase::Intent)]);
}

#[test]
fn a_closing_record_before_the_intent_does_not_close_it() {
    let log = vec![rec("op-1", OpPhase::Done), rec("op-1", OpPhase::Intent)];
    assert_eq!(ids(open_intents(&log)), vec!["op-1"]);
}
