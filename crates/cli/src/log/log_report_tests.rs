//! log_report: "listed" with the restore step, or "empty" with the unfiltered log as next; exit 0 either way; one
//! "records" field holding the encoded lines it is given, unchanged and in order. ZUT: oplog's record types and
//! gitio's Oid::parse are plain data; the encoded lines are literals. That run_log passes each listed op's own log
//! line, byte for byte, is the validation test owed in run_log.rs's header.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

use super::log_report;
use crate::report::{ExitCode, OutValue, Report};

fn record(op_id: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: "k".to_string(),
        kind: "save".to_string(),
        actor: "core/fix-1".to_string(),
        reason: "mr save".to_string(),
        at: 5,
        phase: OpPhase::Intent,
        moves: vec![RefMove {
            refname: "refs/heads/req/core/fix-1".to_string(),
            old: None,
            new: Oid::parse(&"c".repeat(40)).expect("a valid oid"),
        }],
        result: None,
    }
}

#[test]
fn no_record_is_empty_with_the_unfiltered_log_next() {
    assert_eq!(
        log_report(&[], Vec::new()),
        Report {
            verb: "log".to_string(),
            exit: ExitCode::Done,
            outcome: "empty".to_string(),
            lines: vec!["no op in the log matches".to_string()],
            owner: None,
            next: "mr log with no filter lists every op".to_string(),
            fields: vec![("records".to_string(), OutValue::Raw(Vec::new()))],
        }
    );
}

#[test]
fn records_are_listed_with_the_restore_step_next() {
    let (one, two) = (record("op-1"), record("op-2"));
    let encoded = vec!["{\"op_id\":\"op-1\"}".to_string(), "{\"op_id\":\"op-2\"}".to_string()];
    let report = log_report(&[&one, &two], encoded);
    assert_eq!(report.verb, "log");
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "listed");
    assert_eq!(report.owner, None);
    assert_eq!(
        report.next,
        "mr restore <op id> <full ref name>... --actor <name> sets refs back as of one of these ops"
    );
    assert_eq!(report.lines.len(), 4);
    assert_eq!(report.lines[0], "op-1 intent save by core/fix-1 at 5 key k: mr save");
    assert_eq!(report.lines[2], "op-2 intent save by core/fix-1 at 5 key k: mr save");
    assert_eq!(
        report.fields,
        vec![(
            "records".to_string(),
            OutValue::Raw(vec!["{\"op_id\":\"op-1\"}".to_string(), "{\"op_id\":\"op-2\"}".to_string()])
        )]
    );
}
