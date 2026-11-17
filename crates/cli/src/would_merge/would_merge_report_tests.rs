//! would_merge_report: Yes merges (exit 0, next lands the commit) and No conflicts (exit 2, each file on its own
//! line, owned by the change's zone), each saying no ref moved and no commit was written.

use std::path::PathBuf;

use gitio::Oid;
use land::{Squash, WouldMerge};
use oplog::ChangeId;

use super::would_merge_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "1111111111111111111111111111111111111111";
const SOURCE: &str = "2222222222222222222222222222222222222222";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn squash() -> Squash {
    Squash {
        commit: Oid::parse(COMMIT).unwrap(),
        change_id: ChangeId::parse(CHANGE).unwrap(),
        source: Oid::parse(SOURCE).unwrap(),
        requests: vec!["REQ-1".to_string()],
    }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

#[test]
fn yes_merges_and_next_lands_the_commit() {
    assert_eq!(
        would_merge_report(&squash(), &WouldMerge::Yes),
        Report {
            verb: "would-merge".to_string(),
            exit: ExitCode::Done,
            outcome: "merges".to_string(),
            lines: vec![
                "change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) merges \
                 cleanly into integration"
                    .to_string(),
                "no ref moved and no commit was written".to_string(),
            ],
            owner: None,
            next: "mortar land 1111111111111111111111111111111111111111 --actor <name>".to_string(),
            fields: vec![text("change_id", CHANGE), text("commit", COMMIT)],
        }
    );
}

#[test]
fn no_exits_2_listing_each_file_owned_by_the_changes_zone() {
    let files = vec![PathBuf::from("src/a/x"), PathBuf::from("docs/y.md")];
    assert_eq!(
        would_merge_report(&squash(), &WouldMerge::No { files }),
        Report {
            verb: "would-merge".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflict".to_string(),
            lines: vec![
                "change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) conflicts \
                 with integration in:"
                    .to_string(),
                "src/a/x".to_string(),
                "docs/y.md".to_string(),
                "no ref moved and no commit was written".to_string(),
            ],
            owner: Some("the change's zone".to_string()),
            next: "fix the conflict in the change's zone through a request (mortar folder request <zone> <request> \
                   <path> --paths <path>... --actor <name>), then mortar squash <zone> --actor <name>"
                .to_string(),
            fields: vec![
                text("change_id", CHANGE),
                text("commit", COMMIT),
                ("files".to_string(), OutValue::Texts(vec!["src/a/x".to_string(), "docs/y.md".to_string()])),
            ],
        }
    );
}

#[test]
fn no_with_one_file_lists_just_that_file() {
    let report = would_merge_report(&squash(), &WouldMerge::No { files: vec![PathBuf::from("Cargo.lock")] });
    assert_eq!(report.lines.len(), 3);
    assert_eq!(report.lines[1], "Cargo.lock");
    assert_eq!(report.fields[2], ("files".to_string(), OutValue::Texts(vec!["Cargo.lock".to_string()])));
}
