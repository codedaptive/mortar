//! attempt_status_report: status_report with the attempt line second, attempt_next's step and the attempt and
//! branch fields last; exit and outcome stay status_report's.

use gitio::StatusPaths;

use super::{attempt_status_report, status_report, StatusState};
use crate::report::{ExitCode, OutValue};

fn state(saved: bool) -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "task".to_string(),
        paths: StatusPaths::default(),
        saved,
        refused: Vec::new(),
        zone_moved: false,
        joined: false,
        no_work_yet: false,
        paused: None,
        stalled: None,
        overlaps: Vec::new(),
        gaps: Vec::new(),
    }
}

#[test]
fn the_attempt_line_follows_the_request_line() {
    let report = attempt_status_report(2, "try/core/task/2", &state(false));
    assert_eq!(report.lines[0], "request task in zone core");
    assert_eq!(report.lines[1], "attempt folder 2 on branch try/core/task/2");
}

#[test]
fn unsaved_gives_save_and_saved_waits_for_the_harness() {
    assert_eq!(attempt_status_report(1, "try/core/task/1", &state(false)).next, "mr save \"<sentence>\"");
    assert_eq!(
        attempt_status_report(1, "try/core/task/1", &state(true)).next,
        "wait for the harness to keep or drop the attempts"
    );
}

#[test]
fn the_attempt_and_branch_fields_come_last() {
    let fields = attempt_status_report(3, "try/core/task/3", &state(true)).fields;
    assert_eq!(
        fields[fields.len() - 2..],
        [
            ("attempt".to_string(), OutValue::Number(3)),
            ("branch".to_string(), OutValue::Text("try/core/task/3".to_string())),
        ]
    );
}

#[test]
fn exit_outcome_and_owner_are_status_reports() {
    for saved in [false, true] {
        let report = attempt_status_report(1, "try/core/task/1", &state(saved));
        let plain = status_report(&state(saved));
        assert_eq!(report.exit, ExitCode::Done);
        assert_eq!(report.outcome, if saved { "saved" } else { "unsaved" });
        assert_eq!((report.verb, report.outcome, report.owner), (plain.verb, plain.outcome, plain.owner));
        assert_eq!(report.lines.len(), plain.lines.len() + 1);
    }
}
