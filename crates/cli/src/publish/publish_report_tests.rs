//! publish_report: Published (pushed and not, each under push on and push off; one zone, two zones and none),
//! OriginMoved under push on (with and without a moved integration folder) and push off (with one) and
//! NothingToPublish, each with its exit, outcome, lines, owner, next and fields, pushed among them.

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

fn pushed(value: bool) -> (String, OutValue) {
    ("pushed".to_string(), OutValue::Bool(value))
}

fn zones(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn moved() -> PublishAnswer {
    PublishAnswer::OriginMoved { found: Oid::parse(FOUND).unwrap() }
}

#[test]
fn published_names_the_landed_commit_and_where_it_is() {
    let rows = [
        (true, true, "published 1111111111111111111111111111111111111111: origin's main and the local main are at it"),
        (true, false, "published 1111111111111111111111111111111111111111: origin's main and the local main are at it"),
        (
            false,
            false,
            "published 1111111111111111111111111111111111111111: the local main is at it; this publish pushed nothing \
             to origin",
        ),
        (
            false,
            true,
            "published 1111111111111111111111111111111111111111: the local main is at it; this publish pushed nothing \
             to origin",
        ),
    ];
    for (was_pushed, push, first) in rows {
        assert_eq!(
            publish_report(
                &PublishAnswer::Published { origin: Oid::parse(ORIGIN).unwrap() },
                was_pushed,
                push,
                &refs(),
                &zones(&["core"]),
                None
            ),
            Report {
                verb: "publish".to_string(),
                exit: ExitCode::Done,
                outcome: "published".to_string(),
                lines: vec![first.to_string(), "the landing state and the zone's squash ref are removed".to_string()],
                owner: None,
                next: "mr after-publish core --actor <name>".to_string(),
                fields: vec![text("origin", ORIGIN), pushed(was_pushed)],
            },
            "pushed {was_pushed}, push {push}"
        );
    }
}

#[test]
fn published_of_two_zones_or_none_names_after_publish_for_each_zone() {
    let rows = [
        (
            zones(&["core", "peer"]),
            "the landing state and the squash refs of zones core, peer are removed",
            "mr after-publish core --actor <name>, then mr after-publish peer --actor <name>",
        ),
        (
            zones(&[]),
            "the landing state and the zone's squash ref are removed",
            "mr after-publish <zone> --actor <name> for each zone this landing published",
        ),
    ];
    for (batch, second, next) in rows {
        assert_eq!(
            publish_report(
                &PublishAnswer::Published { origin: Oid::parse(ORIGIN).unwrap() },
                true,
                true,
                &refs(),
                &batch,
                None
            ),
            Report {
                verb: "publish".to_string(),
                exit: ExitCode::Done,
                outcome: "published".to_string(),
                lines: vec![
                    "published 1111111111111111111111111111111111111111: origin's main and the local main are at it"
                        .to_string(),
                    second.to_string(),
                ],
                owner: None,
                next: next.to_string(),
                fields: vec![text("origin", ORIGIN), pushed(true)],
            },
            "zones {batch:?}"
        );
    }
}

#[test]
fn origin_moved_exits_1_naming_who_moved_which_origin_and_the_folder() {
    let rows = [
        (
            true,
            Some(Path::new("/work/integration")),
            "someone else moved origin's main to 3333333333333333333333333333333333333333; nothing was published",
            "moved the integration folder /work/integration to 3333333333333333333333333333333333333333",
            "whoever pushed 3333333333333333333333333333333333333333 to origin",
            "land the change's squash again onto 3333333333333333333333333333333333333333: mr land <commit> --actor \
             <name>, then mr publish --actor <name>",
            OutValue::Text("/work/integration".to_string()),
        ),
        (
            true,
            None,
            "someone else moved origin's main to 3333333333333333333333333333333333333333; nothing was published",
            "no integration folder to move",
            "whoever pushed 3333333333333333333333333333333333333333 to origin",
            "land the change's squash again onto 3333333333333333333333333333333333333333: mr land <commit> --actor \
             <name>, then mr publish --actor <name>",
            OutValue::Null,
        ),
        (
            false,
            Some(Path::new("/work/integration")),
            "someone else moved the local main to 3333333333333333333333333333333333333333; nothing was published \
             and nothing was fetched",
            "moved the integration folder /work/integration to 3333333333333333333333333333333333333333",
            "whoever moved the local main to 3333333333333333333333333333333333333333",
            "land the change's squash again onto 3333333333333333333333333333333333333333: mr land <commit> --actor \
             <name>, then mr publish --no-push --actor <name>",
            OutValue::Text("/work/integration".to_string()),
        ),
    ];
    for (push, folder, first, folder_line, owner, next, folder_field) in rows {
        assert_eq!(
            publish_report(&moved(), false, push, &refs(), &[], folder),
            Report {
                verb: "publish".to_string(),
                exit: ExitCode::Refused,
                outcome: "origin-moved".to_string(),
                lines: vec![
                    first.to_string(),
                    "the local main and integration are on 3333333333333333333333333333333333333333; the squash and \
                     its ref are kept"
                        .to_string(),
                    folder_line.to_string(),
                ],
                owner: Some(owner.to_string()),
                next: next.to_string(),
                fields: vec![
                    text("found", FOUND),
                    text("integration", FOUND),
                    ("folder".to_string(), folder_field),
                    pushed(false),
                ],
            },
            "push {push}, folder {folder:?}"
        );
    }
}

#[test]
fn nothing_to_publish_is_done_with_pushed_false() {
    for push in [true, false] {
        assert_eq!(
            publish_report(&PublishAnswer::NothingToPublish, false, push, &refs(), &[], None),
            Report {
                verb: "publish".to_string(),
                exit: ExitCode::Done,
                outcome: "nothing-to-publish".to_string(),
                lines: vec!["no landing is waiting to be published".to_string()],
                owner: None,
                next: "nothing to do".to_string(),
                fields: vec![pushed(false)],
            },
            "push {push}"
        );
    }
}
