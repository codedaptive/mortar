//! back_out_report: BackedOut of one (with and without a moved folder), BackedOut of two, and NothingToBackOut,
//! each with its exit, outcome, lines, next and fields.

use std::path::Path;

use gitio::Oid;
use land::BatchBackOutAnswer;
use oplog::ChangeId;

use super::back_out_report;
use crate::report::{ExitCode, OutValue, Report};

const MAIN: &str = "3333333333333333333333333333333333333333";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const NEWER: &str = "ssssttttuuuuvvvvwwwwxxxxyyyyzzzz";

fn main_oid() -> Oid {
    Oid::parse(MAIN).unwrap()
}

fn change(id: &str) -> ChangeId {
    ChangeId::parse(id).unwrap()
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn texts(name: &str, values: &[&str]) -> (String, OutValue) {
    (name.to_string(), OutValue::Texts(values.iter().map(|value| value.to_string()).collect()))
}

/// A batch of one keeps the single-change lines and next; fields gain backed_out with the one ID. One row with a
/// moved folder, one without.
#[test]
fn backed_out_of_one_keeps_the_single_change_text() {
    let answer = BatchBackOutAnswer::BackedOut { change_id: change(CHANGE), change_ids: vec![change(CHANGE)] };
    let rows: [(Option<&Path>, &str, OutValue); 2] = [
        (
            Some(Path::new("/work/int")),
            "moved the integration folder /work/int to 3333333333333333333333333333333333333333",
            OutValue::Text("/work/int".to_string()),
        ),
        (None, "no integration folder to move", OutValue::Null),
    ];
    for (folder, folder_line, folder_field) in rows {
        assert_eq!(
            back_out_report(&answer, &main_oid(), folder),
            Report {
                verb: "back-out".to_string(),
                exit: ExitCode::Done,
                outcome: "backed-out".to_string(),
                lines: vec![
                    "backed out change kkkkllllmmmmnnnnooooppppqqqqrrrr: integration is back at \
                     3333333333333333333333333333333333333333"
                        .to_string(),
                    "the squash and its ref are kept, with the same change ID".to_string(),
                    folder_line.to_string(),
                ],
                owner: None,
                next: "fix the change in its zone, then mr squash <zone> --actor <name>".to_string(),
                fields: vec![
                    text("change_id", CHANGE),
                    texts("backed_out", &[CHANGE]),
                    text("integration", MAIN),
                    ("folder".to_string(), folder_field),
                ],
            },
            "folder {folder:?}"
        );
    }
}

#[test]
fn backed_out_of_two_names_both_in_landing_order() {
    let answer =
        BatchBackOutAnswer::BackedOut { change_id: change(NEWER), change_ids: vec![change(CHANGE), change(NEWER)] };
    assert_eq!(
        back_out_report(&answer, &main_oid(), Some(Path::new("/work/int"))),
        Report {
            verb: "back-out".to_string(),
            exit: ExitCode::Done,
            outcome: "backed-out".to_string(),
            lines: vec![
                "backed out changes kkkkllllmmmmnnnnooooppppqqqqrrrr, ssssttttuuuuvvvvwwwwxxxxyyyyzzzz (in landing \
                 order): integration is back at 3333333333333333333333333333333333333333"
                    .to_string(),
                "the squashes and their refs are kept, with the same change IDs".to_string(),
                "moved the integration folder /work/int to 3333333333333333333333333333333333333333".to_string(),
            ],
            owner: None,
            next: "land again the squashes that stand, in the same order: mr would-merge <change id> --actor <name> \
                   names each one's commit, then mr land <commit> --actor <name>; or fix a change in its zone, then \
                   mr squash <zone> --actor <name>"
                .to_string(),
            fields: vec![
                text("change_id", NEWER),
                texts("backed_out", &[CHANGE, NEWER]),
                text("integration", MAIN),
                text("folder", "/work/int"),
            ],
        }
    );
}

#[test]
fn nothing_to_back_out_names_origin_and_does_nothing() {
    assert_eq!(
        back_out_report(&BatchBackOutAnswer::NothingToBackOut, &main_oid(), None),
        Report {
            verb: "back-out".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-back-out".to_string(),
            lines: vec![
                "integration is at origin 3333333333333333333333333333333333333333; there is nothing to back out"
                    .to_string()
            ],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![text("integration", MAIN), ("folder".to_string(), OutValue::Null)],
        }
    );
}
