//! The report of `mortar log`.

use oplog::OpRecord;

use super::log_lines;
use crate::report::{ExitCode, OutValue, Report};

/// Exit 0, outcome "listed" with log_lines and next the restore step (S-112: a step that runs now); with no records,
/// outcome "empty" and next the unfiltered log. The one --json field, "records", holds `encoded` unchanged: each
/// record's line as oplog encodes it in the log, in the order of `records`; the caller encodes them (S-114).
pub fn log_report(records: &[&OpRecord], encoded: Vec<String>) -> Report {
    let (outcome, next) = match records {
        [] => ("empty", "mortar log with no filter lists every op"),
        _ => ("listed", "mortar restore <op id> <ref>... sets refs back as of one of these ops"),
    };
    Report {
        verb: "log".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: log_lines(records),
        owner: None,
        next: next.to_string(),
        fields: vec![("records".to_string(), OutValue::Raw(encoded))],
    }
}
