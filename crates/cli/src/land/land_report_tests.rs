//! land_report: Landed in a batch of one (the single-landing lines) and as 2 of 2 (the batch line), Conflicted with
//! no batch and with a batch in flight, Busy and ZoneInBatch, each with its exit, outcome, lines, owner, next and
//! fields.

use std::path::{Path, PathBuf};

use ::land::{BatchLandAnswer, BatchMember};
use gitio::Oid;
use oplog::ChangeId;

use super::land_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "1111111111111111111111111111111111111111";
const SOURCE: &str = "2222222222222222222222222222222222222222";
const MERGE: &str = "3333333333333333333333333333333333333333";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const OTHER: &str = "zzzzyyyyxxxxwwwwvvvvuuuuttttssss";

const LANDED_LINE: &str = "landed change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit \
                           1111111111111111111111111111111111111111): integration is at \
                           3333333333333333333333333333333333333333";
const LANDED_NEXT: &str = "run the integration tests on integration, then mr publish --actor <name> when they pass, \
                           or mr back-out --actor <name> when they fail";
const CONFLICT_LINE: &str = "change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit \
                             1111111111111111111111111111111111111111) conflicts with integration in:";
const CONFLICT_NEXT: &str = "fix the conflict in the change's zone through a request (mr folder request <zone> \
                             <request> <path> --paths <path>... --actor <name>), then mr squash <zone> --actor <name>";

fn squash() -> ::land::Squash {
    ::land::Squash {
        commit: Oid::parse(COMMIT).unwrap(),
        change_id: ChangeId::parse(CHANGE).unwrap(),
        source: Oid::parse(SOURCE).unwrap(),
        requests: vec!["REQ-1".to_string()],
    }
}

fn member(change_id: &str, zone: &str) -> BatchMember {
    BatchMember { change_id: ChangeId::parse(change_id).unwrap(), zone: zone.to_string() }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn texts(name: &str, values: &[&str]) -> (String, OutValue) {
    (name.to_string(), OutValue::Texts(values.iter().map(|value| value.to_string()).collect()))
}

fn landed() -> BatchLandAnswer {
    BatchLandAnswer::Landed { integration: Oid::parse(MERGE).unwrap() }
}

fn conflicted() -> BatchLandAnswer {
    BatchLandAnswer::Conflicted { files: vec![PathBuf::from("src/a/x"), PathBuf::from("src/b/y")] }
}

fn landed_report(lines: &[&str], folder: OutValue, batch: &[&str]) -> Report {
    Report {
        verb: "land".to_string(),
        exit: ExitCode::Done,
        outcome: "landed".to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: None,
        next: LANDED_NEXT.to_string(),
        fields: vec![
            text("change_id", CHANGE),
            text("commit", COMMIT),
            text("integration", MERGE),
            ("folder".to_string(), folder),
            texts("batch", batch),
        ],
    }
}

#[test]
fn landed_in_a_batch_of_one_keeps_todays_lines_and_names_itself_as_the_batch() {
    let folder_line = "moved the integration folder /work/integration to 3333333333333333333333333333333333333333";
    let rows: [(&str, Vec<BatchMember>, Option<&Path>, Report); 2] = [
        (
            "no batch read, a folder moved",
            vec![],
            Some(Path::new("/work/integration")),
            landed_report(&[LANDED_LINE, folder_line], OutValue::Text("/work/integration".to_string()), &[CHANGE]),
        ),
        (
            "a batch of this squash alone, no folder",
            vec![member(CHANGE, "core")],
            None,
            landed_report(&[LANDED_LINE, "no integration folder to move"], OutValue::Null, &[CHANGE]),
        ),
    ];
    for (name, batch, folder, expected) in rows {
        assert_eq!(land_report(&squash(), &landed(), &batch, folder), expected, "{name}");
    }
}

#[test]
fn landed_second_of_two_inserts_its_place_in_the_batch_and_lists_the_batch() {
    let batch = [member(OTHER, "core"), member(CHANGE, "peer")];
    assert_eq!(
        land_report(&squash(), &landed(), &batch, None),
        landed_report(
            &[
                LANDED_LINE,
                "landed 2 of the batch, in landing order: zzzzyyyyxxxxwwwwvvvvuuuuttttssss, \
                 kkkkllllmmmmnnnnooooppppqqqqrrrr",
                "no integration folder to move",
            ],
            OutValue::Null,
            &[OTHER, CHANGE],
        )
    );
}

#[test]
fn conflicted_with_no_batch_exits_1_listing_each_file_as_today_with_an_empty_batch() {
    assert_eq!(
        land_report(&squash(), &conflicted(), &[], None),
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflict".to_string(),
            lines: vec![
                CONFLICT_LINE.to_string(),
                "src/a/x".to_string(),
                "src/b/y".to_string(),
                "integration was not moved and no landing state was written".to_string(),
            ],
            owner: Some("the change's zone".to_string()),
            next: CONFLICT_NEXT.to_string(),
            fields: vec![
                text("change_id", CHANGE),
                text("commit", COMMIT),
                texts("files", &["src/a/x", "src/b/y"]),
                texts("batch", &[]),
            ],
        }
    );
}

#[test]
fn conflicted_with_a_batch_in_flight_names_the_unchanged_batch_and_says_it_is_kept() {
    assert_eq!(
        land_report(&squash(), &conflicted(), &[member(OTHER, "core")], None),
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflict".to_string(),
            lines: vec![
                CONFLICT_LINE.to_string(),
                "src/a/x".to_string(),
                "src/b/y".to_string(),
                "integration was not moved and the batch of 1 is unchanged: zzzzyyyyxxxxwwwwvvvvuuuuttttssss"
                    .to_string(),
            ],
            owner: Some("the change's zone".to_string()),
            next: format!(
                "{CONFLICT_NEXT}; the batch is kept: land another zone's squash onto it with mr land <commit> \
                 --actor <name> (mr would-merge <change id> --actor <name> names a squash's commit)"
            ),
            fields: vec![
                text("change_id", CHANGE),
                text("commit", COMMIT),
                texts("files", &["src/a/x", "src/b/y"]),
                texts("batch", &[OTHER]),
            ],
        }
    );
}

#[test]
fn busy_exits_4_owned_by_the_other_changes_landing() {
    let busy = BatchLandAnswer::Busy { change_id: ChangeId::parse(OTHER).unwrap() };
    assert_eq!(
        land_report(&squash(), &busy, &[], None),
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
            next: "finish that landing first (mr publish --actor <name>, or mr back-out --actor <name>), then \
                   mr land 1111111111111111111111111111111111111111 --actor <name>"
                .to_string(),
            fields: vec![text("change_id", CHANGE), text("commit", COMMIT), text("busy_with", OTHER)],
        }
    );
}

#[test]
fn zone_in_batch_is_refused_naming_the_member_that_holds_the_zone() {
    let answer = BatchLandAnswer::ZoneInBatch { zone: "core".to_string(), change_id: ChangeId::parse(OTHER).unwrap() };
    assert_eq!(
        land_report(&squash(), &answer, &[], None),
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Refused,
            outcome: "zone-in-batch".to_string(),
            lines: vec![
                "zone core is already in the batch as change zzzzyyyyxxxxwwwwvvvvuuuuttttssss; change \
                 kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) was not landed"
                    .to_string(),
                "integration and the batch were not moved".to_string(),
            ],
            owner: Some("change zzzzyyyyxxxxwwwwvvvvuuuuttttssss's landing".to_string()),
            next: "publish or back out the batch first (mr publish --actor <name>, or mr back-out --actor <name>), \
                   then mr land 1111111111111111111111111111111111111111 --actor <name>"
                .to_string(),
            fields: vec![
                text("change_id", CHANGE),
                text("commit", COMMIT),
                text("zone", "core"),
                text("member", OTHER),
            ],
        }
    );
}
