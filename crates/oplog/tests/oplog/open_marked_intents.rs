//! Joins open_marked_intents -> open_intents_in on marked logs built from real MarkedRecord values: the open
//! intents of a marked log come back whole and in log order, remote-marked ones included; intents closed by a
//! later Done or Failed record are left out.

use oplog::{open_marked_intents, MarkedRecord, OpPhase, OpRecord, OpResult};

fn marked(op_id: &str, phase: OpPhase, remote: Option<&str>) -> MarkedRecord {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved { refs: Vec::new() }),
    };
    MarkedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("key-{op_id}"),
            kind: "land".to_string(),
            actor: "agent-a".to_string(),
            reason: "squash lands".to_string(),
            at: 1763030400,
            phase,
            moves: vec![],
            result,
        },
        values: vec![],
        remote: remote.map(str::to_string),
    }
}

fn op_ids(open: Vec<&MarkedRecord>) -> Vec<&str> {
    open.into_iter().map(|m| m.record.op_id.as_str()).collect()
}

#[test]
fn an_empty_log_has_no_open_intents() {
    assert_eq!(open_marked_intents(&[]), Vec::<&MarkedRecord>::new());
}

#[test]
fn intents_closed_by_a_later_done_or_failed_are_left_out() {
    let log = vec![
        marked("op-1", OpPhase::Intent, None),
        marked("op-2", OpPhase::Intent, None),
        marked("op-3", OpPhase::Intent, None),
        marked("op-1", OpPhase::Done, None),
        marked("op-2", OpPhase::Failed, None),
    ];
    assert_eq!(op_ids(open_marked_intents(&log)), vec!["op-3"]);
}

#[test]
fn a_remote_marked_intent_is_returned_whole() {
    let mut intent = marked("op-1", OpPhase::Intent, Some("origin"));
    intent.values = vec![("refs/heads/main".to_string(), "abc123".to_string())];
    let expected = MarkedRecord {
        record: OpRecord {
            op_id: "op-1".to_string(),
            key: "key-op-1".to_string(),
            kind: "land".to_string(),
            actor: "agent-a".to_string(),
            reason: "squash lands".to_string(),
            at: 1763030400,
            phase: OpPhase::Intent,
            moves: vec![],
            result: None,
        },
        values: vec![("refs/heads/main".to_string(), "abc123".to_string())],
        remote: Some("origin".to_string()),
    };
    assert_eq!(open_marked_intents(&[intent]), vec![&expected]);
}

#[test]
fn open_intents_come_back_in_log_order_remote_ones_included() {
    let log = vec![
        marked("op-3", OpPhase::Intent, Some("origin")),
        marked("op-1", OpPhase::Intent, None),
        marked("op-4", OpPhase::Intent, Some("upstream")),
        marked("op-2", OpPhase::Intent, None),
    ];
    let open = open_marked_intents(&log);
    assert_eq!(op_ids(open.clone()), vec!["op-3", "op-1", "op-4", "op-2"]);
    assert_eq!(
        open.iter().map(|m| m.remote.as_deref()).collect::<Vec<_>>(),
        vec![Some("origin"), None, Some("upstream"), None]
    );
}
