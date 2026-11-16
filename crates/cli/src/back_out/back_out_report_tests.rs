//! back_out_report: BackedOut with and without a moved folder, and NothingToBackOut, each with its exit, outcome,
//! lines, next and fields.

use std::path::Path;

use gitio::Oid;
use land::BackOutAnswer;
use oplog::ChangeId;

use super::back_out_report;
use crate::report::{ExitCode, OutValue, Report};

const MAIN: &str = "3333333333333333333333333333333333333333";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn main_oid() -> Oid {
    Oid::parse(MAIN).unwrap()
}

fn backed_out() -> BackOutAnswer {
    BackOutAnswer::BackedOut { change_id: ChangeId::parse(CHANGE).unwrap() }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

#[test]
fn backed_out_with_a_moved_folder_names_change_integration_and_folder() {
    assert_eq!(
        back_out_report(&backed_out(), &main_oid(), Some(Path::new("/work/int"))),
        Report {
            verb: "back-out".to_string(),
            exit: ExitCode::Done,
            outcome: "backed-out".to_string(),
            lines: vec![
                "backed out change kkkkllllmmmmnnnnooooppppqqqqrrrr: integration is back at \
                 3333333333333333333333333333333333333333"
                    .to_string(),
                "the squash and its ref are kept, with the same change ID".to_string(),
                "moved the integration folder /work/int to 3333333333333333333333333333333333333333".to_string(),
            ],
            owner: None,
            next: "fix the change in its zone, then mortar squash <zone> --actor <name>".to_string(),
            fields: vec![text("change_id", CHANGE), text("integration", MAIN), text("folder", "/work/int")],
        }
    );
}

#[test]
fn backed_out_with_no_folder_says_so_and_folder_is_null() {
    let report = back_out_report(&backed_out(), &main_oid(), None);
    assert_eq!(report.lines[2], "no integration folder to move");
    assert_eq!(
        report.fields,
        vec![text("change_id", CHANGE), text("integration", MAIN), ("folder".to_string(), OutValue::Null)]
    );
}

#[test]
fn nothing_to_back_out_names_origin_and_does_nothing() {
    assert_eq!(
        back_out_report(&BackOutAnswer::NothingToBackOut, &main_oid(), None),
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
