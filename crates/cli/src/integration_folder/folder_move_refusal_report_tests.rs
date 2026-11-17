//! folder_move_refusal_report: exit 1 "folder-not-moved" naming the folder, both tips and the owner's step in words.

use std::path::Path;

use gitio::Oid;

use super::folder_move_refusal_report;
use crate::report::{ExitCode, OutValue, Report};

#[test]
fn a_folder_that_could_not_move_is_refused_with_the_owners_step_in_words() {
    let old = Oid::parse("1111111111111111111111111111111111111111").expect("a valid oid");
    let new = Oid::parse("2222222222222222222222222222222222222222").expect("a valid oid");
    let report = folder_move_refusal_report(
        "back-out",
        "integration is back at 2222222222222222222222222222222222222222",
        Path::new("/r/integration"),
        &old,
        &new,
        "/r/integration was not moved: local changes are in the way at a.txt",
    );
    assert_eq!(
        report,
        Report {
            verb: "back-out".to_string(),
            exit: ExitCode::Refused,
            outcome: "folder-not-moved".to_string(),
            lines: vec![
                "integration is back at 2222222222222222222222222222222222222222".to_string(),
                "the integration folder /r/integration could not move from 1111111111111111111111111111111111111111 \
                 to 2222222222222222222222222222222222222222: /r/integration was not moved: local changes are in \
                 the way at a.txt"
                    .to_string(),
            ],
            owner: Some("the integration folder /r/integration".to_string()),
            next: "clear what stops the integration folder /r/integration from moving, then bring its files and \
                   index to 2222222222222222222222222222222222222222, the commit integration now points at; \
                   integration itself has already moved"
                .to_string(),
            fields: vec![
                ("folder".to_string(), OutValue::Text("/r/integration".to_string())),
                ("old".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
                ("new".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
            ],
        }
    );
}
