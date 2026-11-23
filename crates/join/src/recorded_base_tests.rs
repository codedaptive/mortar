//! recorded_base: Recorded for a join-rebase op and a join-zone-request op in any phase, the first of repeated values
//! taken; NotRecorded for an op with no values, an older resolve op with only folder_tree and join_state, an op with
//! only one of the two keys, and another kind carrying both; Unreadable, naming the key, for a value that does not
//! decode. Ops are built as data; the recorded texts are written out by hand.

use crate::partner::Partner;
use crate::recorded_base::recorded_base;
use crate::recorded_base_type::RecordedBase;
use crate::request_base::RequestBase;
use gitio::Oid;
use oplog::{OpPhase, OpRecord, ValuedRecord};

const BEFORE: &str = "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[]}";
const AFTER: &str = concat!(
    "{\"base\":\"2222222222222222222222222222222222222222\",\"partners\":[",
    "{\"zone\":\"east\",\"tip\":\"3333333333333333333333333333333333333333\"}]}"
);
const OTHER: &str = "{\"base\":\"4444444444444444444444444444444444444444\",\"partners\":[]}";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn recorded() -> RecordedBase {
    RecordedBase::Recorded {
        before: RequestBase { base: oid("1111111111111111111111111111111111111111"), partners: vec![] },
        after: RequestBase {
            base: oid("2222222222222222222222222222222222222222"),
            partners: vec![Partner { zone: "east".to_string(), tip: oid("3333333333333333333333333333333333333333") }],
        },
    }
}

fn op(kind: &str, phase: OpPhase, values: &[(&str, &str)]) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: "op-7".to_string(),
            key: format!("{kind}:refs/heads/request/r1:op-7"),
            kind: kind.to_string(),
            actor: "42@host".to_string(),
            reason: "join".to_string(),
            at: 9,
            phase,
            moves: vec![],
            result: None,
        },
        values: values.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect(),
    }
}

#[test]
fn recorded_base_reads_the_two_values_of_a_join_op() {
    let both = [("base_before", BEFORE), ("base_after", AFTER)];
    let tree = ("folder_tree", "9999999999999999999999999999999999999999");
    let rows = [
        ("join-rebase, done", op("join-rebase", OpPhase::Done, &both), recorded()),
        ("join-zone-request, done", op("join-zone-request", OpPhase::Done, &both), recorded()),
        ("join-rebase, intent", op("join-rebase", OpPhase::Intent, &both), recorded()),
        ("join-zone-request, failed", op("join-zone-request", OpPhase::Failed, &both), recorded()),
        (
            "a resolve's final op, the bases after folder_tree and join_state",
            op("join-rebase", OpPhase::Done, &[tree, ("join_state", "{}"), both[0], both[1]]),
            recorded(),
        ),
        (
            "the first of repeated values taken",
            op("join-rebase", OpPhase::Done, &[both[0], both[1], ("base_before", OTHER), ("base_after", OTHER)]),
            recorded(),
        ),
        ("no values", op("join-rebase", OpPhase::Done, &[]), RecordedBase::NotRecorded),
        (
            "an older resolve op with only folder_tree and join_state",
            op("join-rebase", OpPhase::Done, &[tree, ("join_state", "{}")]),
            RecordedBase::NotRecorded,
        ),
        ("only base_before", op("join-rebase", OpPhase::Done, &[both[0]]), RecordedBase::NotRecorded),
        ("only base_after", op("join-zone-request", OpPhase::Done, &[both[1]]), RecordedBase::NotRecorded),
        ("another kind carrying both", op("undo", OpPhase::Done, &both), RecordedBase::NotRecorded),
        (
            "a garbled base_before",
            op("join-rebase", OpPhase::Done, &[("base_before", "{\"base\""), both[1]]),
            RecordedBase::Unreadable { reason: "base_before: the text ends before the base is complete".to_string() },
        ),
        (
            "a garbled base_after",
            op("join-zone-request", OpPhase::Done, &[both[0], ("base_after", "[]")]),
            RecordedBase::Unreadable {
                reason: "base_after: the base is not an object with exactly the members base then partners".to_string(),
            },
        ),
    ];
    for (case, op, expected) in rows {
        assert_eq!(recorded_base(&op), expected, "{case}");
    }
}
