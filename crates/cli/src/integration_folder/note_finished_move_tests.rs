//! note_finished_move: no finished move leaves the report as it was; a finished move puts its one line first and
//! keeps everything else.

use std::path::PathBuf;

use gitio::Oid;

use super::note_finished_move;
use crate::integration_folder::FinishedMove;
use crate::report::{ExitCode, OutValue, Report};

const OLD: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";

fn report() -> Report {
    Report {
        verb: "back-out".to_string(),
        exit: ExitCode::Done,
        outcome: "nothing-to-back-out".to_string(),
        lines: vec!["no landing is waiting to be backed out".to_string(), "second line".to_string()],
        owner: Some("the op log".to_string()),
        next: "nothing to do".to_string(),
        fields: vec![
            ("integration".to_string(), OutValue::Text(TIP.to_string())),
            ("folder".to_string(), OutValue::Null),
        ],
    }
}

fn finished() -> FinishedMove {
    FinishedMove {
        folder: PathBuf::from("/work/integration"),
        old: Oid::parse(OLD).unwrap(),
        tip: Oid::parse(TIP).unwrap(),
    }
}

#[test]
fn no_finished_move_leaves_the_report_as_it_was() {
    assert_eq!(note_finished_move(report(), None), report());
}

#[test]
fn a_finished_move_puts_its_line_first_and_keeps_the_rest() {
    assert_eq!(
        note_finished_move(report(), Some(&finished())),
        Report {
            verb: "back-out".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-back-out".to_string(),
            lines: vec![
                "first finished a stopped move: the integration folder /work/integration moved from \
                 1111111111111111111111111111111111111111 to 2222222222222222222222222222222222222222"
                    .to_string(),
                "no landing is waiting to be backed out".to_string(),
                "second line".to_string(),
            ],
            owner: Some("the op log".to_string()),
            next: "nothing to do".to_string(),
            fields: vec![
                ("integration".to_string(), OutValue::Text(TIP.to_string())),
                ("folder".to_string(), OutValue::Null),
            ],
        }
    );
}

#[test]
fn a_finished_move_on_a_report_with_no_lines_becomes_its_only_line() {
    let failure = Report {
        verb: "land".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: Vec::new(),
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    };
    let noted = note_finished_move(failure, Some(&finished()));
    assert_eq!(
        noted.lines,
        vec![
            "first finished a stopped move: the integration folder /work/integration moved from \
             1111111111111111111111111111111111111111 to 2222222222222222222222222222222222222222"
                .to_string()
        ]
    );
    assert_eq!(noted.verb, "land");
    assert_eq!(noted.exit, ExitCode::Failure);
    assert_eq!(noted.outcome, "failed");
    assert_eq!(noted.owner, None);
    assert_eq!(noted.next, "repeat the same command");
    assert_eq!(noted.fields, Vec::new());
}
