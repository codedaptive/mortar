//! land_report: Landed (with and without a moved integration folder), Conflicted and Busy, each with its exit,
//! outcome, lines, owner, next and fields.

use std::path::{Path, PathBuf};

use gitio::Oid;
use oplog::ChangeId;

use super::land_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "1111111111111111111111111111111111111111";
const SOURCE: &str = "2222222222222222222222222222222222222222";
const MERGE: &str = "3333333333333333333333333333333333333333";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const OTHER: &str = "zzzzyyyyxxxxwwwwvvvvuuuuttttssss";

fn squash() -> ::land::Squash {
    ::land::Squash {
        commit: Oid::parse(COMMIT).unwrap(),
        change_id: ChangeId::parse(CHANGE).unwrap(),
        source: Oid::parse(SOURCE).unwrap(),
        requests: vec!["REQ-1".to_string()],
    }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn landed() -> ::land::LandAnswer {
    ::land::LandAnswer::Landed { integration: Oid::parse(MERGE).unwrap() }
}

#[test]
fn landed_with_a_folder_names_integration_and_the_moved_folder() {
    assert_eq!(
        land_report(&squash(), &landed(), Some(Path::new("/work/integration"))),
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Done,
            outcome: "landed".to_string(),
            lines: vec![
                "landed change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111): \
                 integration is at 3333333333333333333333333333333333333333"
                    .to_string(),
                "moved the integration folder /work/integration to 3333333333333333333333333333333333333333"
                    .to_string(),
            ],
            owner: None,
            next: "run the integration tests on integration, then mortar publish --actor <name> when they pass, or \
                   mortar back-out --actor <name> when they fail"
                .to_string(),
            fields: vec![
                text("change_id", CHANGE),
                text("commit", COMMIT),
                text("integration", MERGE),
                text("folder", "/work/integration"),
            ],
        }
    );
}

#[test]
fn landed_without_a_folder_says_none_and_folder_is_null() {
    let report = land_report(&squash(), &landed(), None);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "landed");
    assert_eq!(report.lines[1], "no integration folder to move");
    assert_eq!(
        report.fields,
        vec![
            text("change_id", CHANGE),
            text("commit", COMMIT),
            text("integration", MERGE),
            ("folder".to_string(), OutValue::Null),
        ]
    );
}

#[test]
fn conflicted_exits_2_listing_each_file_owned_by_the_changes_zone() {
    let files = vec![PathBuf::from("src/a/x"), PathBuf::from("src/b/y")];
    assert_eq!(
        land_report(&squash(), &::land::LandAnswer::Conflicted { files }, None),
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflict".to_string(),
            lines: vec![
                "change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) conflicts \
                 with integration in:"
                    .to_string(),
                "src/a/x".to_string(),
                "src/b/y".to_string(),
                "integration was not moved and no landing state was written".to_string(),
            ],
            owner: Some("the change's zone".to_string()),
            next: "fix the conflict in the change's zone through a request (mortar folder request <zone> <request> \
                   <path> --paths <path>... --actor <name>), then mortar squash <zone> --actor <name>"
                .to_string(),
            fields: vec![
                text("change_id", CHANGE),
                text("commit", COMMIT),
                ("files".to_string(), OutValue::Texts(vec!["src/a/x".to_string(), "src/b/y".to_string()])),
            ],
        }
    );
}

#[test]
fn busy_exits_3_owned_by_the_other_changes_landing() {
    let busy = ::land::LandAnswer::Busy { change_id: ChangeId::parse(OTHER).unwrap() };
    assert_eq!(
        land_report(&squash(), &busy, None),
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Busy,
            outcome: "busy".to_string(),
            lines: vec![
                "change zzzzyyyyxxxxwwwwvvvvuuuuttttssss's landing is in flight on integration; change \
                 kkkkllllmmmmnnnnooooppppqqqqrrrr was not landed"
                    .to_string()
            ],
            owner: Some("change zzzzyyyyxxxxwwwwvvvvuuuuttttssss's landing".to_string()),
            next: "finish that landing first (mortar publish --actor <name>, or mortar back-out --actor <name>), then \
                   mortar land 1111111111111111111111111111111111111111 --actor <name>"
                .to_string(),
            fields: vec![text("change_id", CHANGE), text("commit", COMMIT), text("busy_with", OTHER)],
        }
    );
}
