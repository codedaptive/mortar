//! squash_report: Made, Same, Nothing and Conflicted, each with its exit, outcome, lines, next and fields.

use std::path::PathBuf;

use gitio::Oid;
use land::{Squash, SquashAnswer};
use oplog::ChangeId;

use super::squash_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "1111111111111111111111111111111111111111";
const SOURCE: &str = "2222222222222222222222222222222222222222";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn squash(requests: &[&str]) -> Squash {
    Squash {
        commit: Oid::parse(COMMIT).unwrap(),
        change_id: ChangeId::parse(CHANGE).unwrap(),
        source: Oid::parse(SOURCE).unwrap(),
        requests: requests.iter().map(|request| request.to_string()).collect(),
    }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn squash_fields(requests: &[&str]) -> Vec<(String, OutValue)> {
    vec![
        text("zone", "a"),
        text("change_id", CHANGE),
        text("commit", COMMIT),
        text("source", SOURCE),
        ("requests".to_string(), OutValue::Texts(requests.iter().map(|request| request.to_string()).collect())),
    ]
}

#[test]
fn made_is_squashed_naming_change_commit_tip_and_requests() {
    assert_eq!(
        squash_report("a", &SquashAnswer::Made(squash(&["REQ-1", "REQ-2"]))),
        Report {
            verb: "squash".to_string(),
            exit: ExitCode::Done,
            outcome: "squashed".to_string(),
            lines: vec![
                "squashed zone a as change kkkkllllmmmmnnnnooooppppqqqqrrrr: commit \
                 1111111111111111111111111111111111111111"
                    .to_string(),
                "it squashes zone tip 2222222222222222222222222222222222222222".to_string(),
                "requests: REQ-1, REQ-2".to_string(),
            ],
            owner: None,
            next: "mr would-merge 1111111111111111111111111111111111111111 --actor <name>, then hand change \
                   kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) to review"
                .to_string(),
            fields: squash_fields(&["REQ-1", "REQ-2"]),
        }
    );
}

#[test]
fn same_is_already_squashed_with_nothing_written() {
    assert_eq!(
        squash_report("a", &SquashAnswer::Same(squash(&[]))),
        Report {
            verb: "squash".to_string(),
            exit: ExitCode::Done,
            outcome: "already-squashed".to_string(),
            lines: vec![
                "zone a's squash is already commit 1111111111111111111111111111111111111111 (change \
                 kkkkllllmmmmnnnnooooppppqqqqrrrr); nothing was written"
                    .to_string(),
                "requests: none named".to_string(),
            ],
            owner: None,
            next: "mr would-merge 1111111111111111111111111111111111111111 --actor <name>, then hand change \
                   kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) to review"
                .to_string(),
            fields: squash_fields(&[]),
        }
    );
}

#[test]
fn nothing_is_nothing_to_squash_naming_the_tip() {
    assert_eq!(
        squash_report("b", &SquashAnswer::Nothing { source: Oid::parse(SOURCE).unwrap() }),
        Report {
            verb: "squash".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-squash".to_string(),
            lines: vec![
                "zone b at 2222222222222222222222222222222222222222 holds nothing origin lacks; nothing was written"
                    .to_string()
            ],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![text("zone", "b"), text("source", SOURCE)],
        }
    );
}

#[test]
fn conflicted_exits_2_listing_each_file() {
    let files = vec![PathBuf::from("src/a/x"), PathBuf::from("src/a/y")];
    assert_eq!(
        squash_report("a", &SquashAnswer::Conflicted { files }),
        Report {
            verb: "squash".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflict".to_string(),
            lines: vec![
                "zone a conflicts with origin in:".to_string(),
                "src/a/x".to_string(),
                "src/a/y".to_string(),
                "nothing was written and no ref moved".to_string(),
            ],
            owner: Some("zone a".to_string()),
            next: "bring zone a up to date with origin through a request (mr folder request a <request> <path> \
                   --paths <path>... --actor <name>), then mr squash a --actor <name>"
                .to_string(),
            fields: vec![
                text("zone", "a"),
                ("files".to_string(), OutValue::Texts(vec!["src/a/x".to_string(), "src/a/y".to_string()])),
            ],
        }
    );
}
