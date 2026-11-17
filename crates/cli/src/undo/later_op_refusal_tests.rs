//! later_op_refusal over histories given as data: the chosen op is still the last done op on its refs, or a later
//! done op that moved one of them is named with its actor, and nothing is undone (S-73, S-142).

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

use super::later_op_refusal;
use crate::report::{ExitCode, Report};

const REQ: &str = "refs/heads/req/orc/WO-cli-4";
const SNAP: &str = "refs/mortar/snap/orc/WO-cli-4";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn op(op_id: &str, actor: &str, phase: OpPhase, refs: &[&str]) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "save".to_string(),
        actor: actor.to_string(),
        reason: "a step".to_string(),
        at: 100,
        phase,
        moves: refs
            .iter()
            .map(|refname| RefMove { refname: refname.to_string(), old: Some(oid('b')), new: oid('c') })
            .collect(),
        result: (phase == OpPhase::Done).then_some(OpResult::Moved),
    }
}

fn scope() -> Vec<String> {
    vec![REQ.to_string()]
}

#[test]
fn the_chosen_op_still_last_on_its_refs_is_not_refused() {
    let records = [
        op("op-1", "orc/WO-cli-4", OpPhase::Done, &[REQ]),
        op("op-2", "orc/WO-cli-4", OpPhase::Done, &[SNAP]),
    ];
    assert_eq!(later_op_refusal(&records, 0, &scope()), None);
}

#[test]
fn a_later_open_or_failed_op_on_the_same_ref_is_not_refused() {
    let records = [
        op("op-1", "orc/WO-cli-4", OpPhase::Done, &[REQ]),
        op("op-2", "orc/other", OpPhase::Intent, &[REQ]),
        op("op-3", "orc/other", OpPhase::Failed, &[REQ]),
    ];
    assert_eq!(later_op_refusal(&records, 0, &scope()), None);
}

#[test]
fn a_later_done_op_of_another_actor_is_refused_naming_it_and_its_actor() {
    let records = [
        op("op-1", "orc/WO-cli-4", OpPhase::Done, &[REQ]),
        op("op-2", "orc/WO-cli-4", OpPhase::Done, &[SNAP]),
        op("op-3", "orc/other", OpPhase::Done, &[SNAP, REQ]),
    ];
    assert_eq!(
        later_op_refusal(&records, 0, &scope()),
        Some(Report {
            verb: "undo".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "op op-1 cannot be undone: refs/heads/req/orc/WO-cli-4 moved since, by op op-3 of orc/other"
                    .to_string(),
                "nothing was undone; the snapshot taken first is kept".to_string(),
            ],
            owner: Some("orc/other".to_string()),
            next: "ask the zone's coordinator about refs/heads/req/orc/WO-cli-4".to_string(),
            fields: Vec::new(),
        })
    );
}

#[test]
fn of_two_later_ops_the_last_is_named() {
    let records = [
        op("op-1", "orc/WO-cli-4", OpPhase::Done, &[REQ]),
        op("op-2", "orc/first", OpPhase::Done, &[REQ]),
        op("op-3", "orc/second", OpPhase::Done, &[REQ]),
    ];
    let report = later_op_refusal(&records, 0, &scope()).expect("a refusal");
    assert_eq!(report.owner.as_deref(), Some("orc/second"));
    assert_eq!(
        report.lines[0],
        "op op-1 cannot be undone: refs/heads/req/orc/WO-cli-4 moved since, by op op-3 of orc/second"
    );
}
