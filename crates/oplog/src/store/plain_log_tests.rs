//! plain_log: records keep their order without their values, torn_lines are kept, empty stays empty.

use super::{plain_log, OpLog, OpPhase, OpRecord, OpResult, ValuedLog, ValuedRecord};

fn intent() -> OpRecord {
    OpRecord {
        op_id: "1".to_string(),
        key: "k".to_string(),
        kind: "land".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 10,
        phase: OpPhase::Intent,
        moves: Vec::new(),
        result: None,
    }
}

fn done() -> OpRecord {
    OpRecord {
        at: 11,
        phase: OpPhase::Done,
        result: Some(OpResult::Moved),
        ..intent()
    }
}

#[test]
fn records_lose_their_values_and_torn_lines_are_kept() {
    let log = ValuedLog {
        records: vec![
            ValuedRecord {
                record: intent(),
                values: Vec::new(),
            },
            ValuedRecord {
                record: done(),
                values: vec![("folder_tree".to_string(), "t1".to_string())],
            },
        ],
        torn_lines: vec![2, 5],
    };
    assert_eq!(
        plain_log(log),
        OpLog {
            records: vec![intent(), done()],
            torn_lines: vec![2, 5],
        }
    );
}

#[test]
fn an_empty_log_stays_empty() {
    assert_eq!(
        plain_log(ValuedLog::default()),
        OpLog {
            records: Vec::new(),
            torn_lines: Vec::new(),
        }
    );
}
