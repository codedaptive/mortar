//! need_with_next: a read state gives status's next step and nothing else changes; a failed read keeps need's
//! report, adds one line with the failure's reason and gives next "mortar status".

use gitio::StatusPaths;

use super::need_with_next;
use crate::report::{ExitCode, OutValue, Report};
use crate::status::StatusState;

fn need(lines: &[&str]) -> Report {
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Done,
        outcome: "needed".to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: None,
        next: String::new(),
        fields: vec![
            ("path".to_string(), OutValue::Text("kit/b".to_string())),
            ("gap".to_string(), OutValue::Bool(true)),
        ],
    }
}

fn saved_unjoined() -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        paths: StatusPaths::default(),
        saved: true,
        refused: Vec::new(),
        zone_moved: false,
        joined: false,
        paused: None,
        stalled: None,
        overlaps: Vec::new(),
        gaps: Vec::new(),
    }
}

fn failure(outcome: &str, lines: &[&str]) -> Report {
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Failure,
        outcome: outcome.to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: None,
        next: "mortar doctor".to_string(),
        fields: Vec::new(),
    }
}

const ADDED: &str = "added kit/b to this folder";
const GAP: &str = "kit/b is outside the work order: recorded as a gap";

#[test]
fn a_saved_unjoined_state_gives_mortar_join_and_keeps_the_rest() {
    let expected = Report { next: "mortar join".to_string(), ..need(&[ADDED, GAP]) };
    assert_eq!(need_with_next(need(&[ADDED, GAP]), Ok(saved_unjoined())), expected);
}

#[test]
fn a_failed_read_keeps_need_and_adds_the_reason_with_next_mortar_status() {
    let read = Err(failure("failed", &["git rev-parse failed", "exit 128"]));
    let expected = Report {
        lines: vec![
            ADDED.to_string(),
            GAP.to_string(),
            "need is done but its next step could not be read: git rev-parse failed; exit 128".to_string(),
        ],
        next: "mortar status".to_string(),
        ..need(&[ADDED, GAP])
    };
    assert_eq!(need_with_next(need(&[ADDED, GAP]), read), expected);
}

#[test]
fn a_failed_read_with_no_lines_gives_its_outcome_as_the_reason() {
    let expected = Report {
        lines: vec![ADDED.to_string(), "need is done but its next step could not be read: snapshot-refused".to_string()],
        next: "mortar status".to_string(),
        ..need(&[ADDED])
    };
    assert_eq!(need_with_next(need(&[ADDED]), Err(failure("snapshot-refused", &[]))), expected);
}
