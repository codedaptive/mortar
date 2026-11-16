//! publish_report: Published, OriginMoved (with and without a moved integration folder) and NothingToPublish, each
//! with its exit, outcome, lines, owner, next and fields.

use std::path::Path;

use gitio::Oid;
use land::{LandRefs, PublishAnswer};

use super::publish_report;
use crate::report::{ExitCode, OutValue, Report};

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const FOUND: &str = "3333333333333333333333333333333333333333";

fn refs() -> LandRefs {
    LandRefs {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
    }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn moved() -> PublishAnswer {
    PublishAnswer::OriginMoved { found: Oid::parse(FOUND).unwrap() }
}

#[test]
fn published_is_done_naming_the_landed_commit() {
    assert_eq!(
        publish_report(&PublishAnswer::Published { origin: Oid::parse(ORIGIN).unwrap() }, &refs(), None),
        Report {
            verb: "publish".to_string(),
            exit: ExitCode::Done,
            outcome: "published".to_string(),
            lines: vec![
                "published 1111111111111111111111111111111111111111: origin's main and the local main are at it"
                    .to_string(),
                "the landing state and the zone's squash ref are removed".to_string(),
            ],
            owner: None,
            next: "nothing more for this landing; the next landing can start".to_string(),
            fields: vec![text("origin", ORIGIN)],
        }
    );
}

#[test]
fn origin_moved_with_a_folder_exits_1_naming_the_moved_folder() {
    assert_eq!(
        publish_report(&moved(), &refs(), Some(Path::new("/work/integration"))),
        Report {
            verb: "publish".to_string(),
            exit: ExitCode::Refused,
            outcome: "origin-moved".to_string(),
            lines: vec![
                "someone else moved origin's main to 3333333333333333333333333333333333333333; nothing was published"
                    .to_string(),
                "the local main and integration are on 3333333333333333333333333333333333333333; the squash and its \
                 ref are kept"
                    .to_string(),
                "moved the integration folder /work/integration to 3333333333333333333333333333333333333333"
                    .to_string(),
            ],
            owner: Some("whoever pushed 3333333333333333333333333333333333333333 to origin".to_string()),
            next: "land the change's squash again onto 3333333333333333333333333333333333333333, then mortar publish \
                   --actor <name>"
                .to_string(),
            fields: vec![text("found", FOUND), text("integration", FOUND), text("folder", "/work/integration")],
        }
    );
}

#[test]
fn origin_moved_without_a_folder_says_none_and_folder_is_null() {
    let report = publish_report(&moved(), &refs(), None);
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "origin-moved");
    assert_eq!(report.lines[2], "no integration folder to move");
    assert_eq!(
        report.fields,
        vec![text("found", FOUND), text("integration", FOUND), ("folder".to_string(), OutValue::Null)]
    );
}

#[test]
fn nothing_to_publish_is_done_with_no_fields() {
    assert_eq!(
        publish_report(&PublishAnswer::NothingToPublish, &refs(), None),
        Report {
            verb: "publish".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-publish".to_string(),
            lines: vec!["no landing is waiting to be published".to_string()],
            owner: None,
            next: "nothing to do".to_string(),
            fields: Vec::new(),
        }
    );
}
