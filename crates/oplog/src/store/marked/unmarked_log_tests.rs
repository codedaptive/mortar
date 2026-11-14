//! unmarked_log: marks dropped, records and values kept in order, torn lines kept.

use crate::store::{OpPhase, OpRecord, ValuedLog, ValuedRecord};

use super::{unmarked_log, MarkedLog, MarkedRecord};

fn record(op_id: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: "k".to_string(),
        kind: "push".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 1,
        phase: OpPhase::Intent,
        moves: Vec::new(),
        result: None,
    }
}

fn pair(n: &str, v: &str) -> (String, String) {
    (n.to_string(), v.to_string())
}

#[test]
fn marks_are_dropped_and_the_rest_kept_in_order() {
    let log = MarkedLog {
        records: vec![
            MarkedRecord {
                record: record("2"),
                values: vec![pair("n", "v")],
                remote: Some("origin".to_string()),
            },
            MarkedRecord {
                record: record("1"),
                values: Vec::new(),
                remote: None,
            },
            MarkedRecord {
                record: record("3"),
                values: vec![pair("a", "1"), pair("b", "")],
                remote: Some("up".to_string()),
            },
        ],
        torn_lines: vec![2, 5],
    };
    assert_eq!(
        unmarked_log(log),
        ValuedLog {
            records: vec![
                ValuedRecord {
                    record: record("2"),
                    values: vec![pair("n", "v")],
                },
                ValuedRecord {
                    record: record("1"),
                    values: Vec::new(),
                },
                ValuedRecord {
                    record: record("3"),
                    values: vec![pair("a", "1"), pair("b", "")],
                },
            ],
            torn_lines: vec![2, 5],
        }
    );
}

#[test]
fn an_empty_log_stays_empty() {
    assert_eq!(unmarked_log(MarkedLog::default()), ValuedLog::default());
}
