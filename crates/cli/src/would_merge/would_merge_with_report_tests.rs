//! would_merge_with_report: Clean merges (exit 0, a line per step, next lands each commit in order) and Conflicted
//! (exit 2, naming the step and, past step 1, the one before it, each file on its own line, owned by the change's
//! zone, with a step object), each saying no ref moved and any merge commit the trial wrote is reached by nothing.

use std::path::PathBuf;

use gitio::Oid;
use land::{Squash, WouldMergeWith};
use oplog::ChangeId;

use super::would_merge_with_report;
use crate::report::{ExitCode, OutValue, Report};

const A_COMMIT: &str = "1111111111111111111111111111111111111111";
const B_COMMIT: &str = "3333333333333333333333333333333333333333";
const C_COMMIT: &str = "4444444444444444444444444444444444444444";
const A_ID: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const B_ID: &str = "llllllllllllllllllllllllllllllll";
const C_ID: &str = "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm";
const NOTHING_REACHED: &str = "no ref moved; any merge commit the trial wrote is reached by nothing";
const CONFLICT_NEXT: &str = "fix the conflict in the change's zone through a request (mr folder request <zone> \
                             <request> <path> --paths <path>... --actor <name>), then mr squash <zone> --actor <name>";

fn squash(commit: &str, id: &str) -> Squash {
    Squash {
        commit: Oid::parse(commit).unwrap(),
        change_id: ChangeId::parse(id).unwrap(),
        source: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
        requests: vec!["REQ-1".to_string()],
    }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(ToString::to_string).collect()
}

fn merges(lines: &[&str], next: &str) -> Report {
    Report {
        verb: "would-merge".to_string(),
        exit: ExitCode::Done,
        outcome: "merges".to_string(),
        lines: strings(lines),
        owner: None,
        next: next.to_string(),
        fields: vec![text("change_id", A_ID), text("commit", A_COMMIT)],
    }
}

fn conflict(lines: &[&str], files: &[&str], number: u64, commit: &str) -> Report {
    Report {
        verb: "would-merge".to_string(),
        exit: ExitCode::Conflict,
        outcome: "conflict".to_string(),
        lines: strings(lines),
        owner: Some("the change's zone".to_string()),
        next: CONFLICT_NEXT.to_string(),
        fields: vec![
            text("change_id", A_ID),
            text("commit", A_COMMIT),
            ("files".to_string(), OutValue::Texts(strings(files))),
            (
                "step".to_string(),
                OutValue::Object(vec![("number".to_string(), OutValue::Number(number)), text("commit", commit)]),
            ),
        ],
    }
}

fn conflicted(step: usize, commit: &str, files: &[&str]) -> WouldMergeWith {
    WouldMergeWith::Conflicted { step, commit: Oid::parse(commit).unwrap(), files: files.iter().map(PathBuf::from).collect() }
}

#[test]
fn reports_each_step_of_the_answer() {
    let both = vec![squash(B_COMMIT, B_ID), squash(C_COMMIT, C_ID)];
    let rows: Vec<(&str, Vec<Squash>, WouldMergeWith, Report)> = vec![
        (
            "clean, no further squash",
            vec![],
            WouldMergeWith::Clean,
            merges(
                &[
                    "change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) merges \
                     cleanly into integration",
                    NOTHING_REACHED,
                ],
                "mr land 1111111111111111111111111111111111111111 --actor <name>",
            ),
        ),
        (
            "clean, two further squashes",
            both.clone(),
            WouldMergeWith::Clean,
            merges(
                &[
                    "change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) merges \
                     cleanly into integration",
                    "then change llllllllllllllllllllllllllllllll (commit 3333333333333333333333333333333333333333) \
                     merges cleanly onto step 1",
                    "then change mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm (commit 4444444444444444444444444444444444444444) \
                     merges cleanly onto step 2",
                    NOTHING_REACHED,
                ],
                "mr land 1111111111111111111111111111111111111111 --actor <name>, then mr land \
                 3333333333333333333333333333333333333333 --actor <name>, then mr land \
                 4444444444444444444444444444444444444444 --actor <name>",
            ),
        ),
        (
            "step 1 conflicts with integration",
            both.clone(),
            conflicted(1, A_COMMIT, &["src/a/x", "docs/y.md"]),
            conflict(
                &[
                    "step 1: change kkkkllllmmmmnnnnooooppppqqqqrrrr (commit 1111111111111111111111111111111111111111) \
                     conflicts with integration in:",
                    "src/a/x",
                    "docs/y.md",
                    NOTHING_REACHED,
                ],
                &["src/a/x", "docs/y.md"],
                1,
                A_COMMIT,
            ),
        ),
        (
            "step 3 conflicts after step 2",
            both,
            conflicted(3, C_COMMIT, &["Cargo.lock"]),
            conflict(
                &[
                    "step 3: change mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm (commit 4444444444444444444444444444444444444444) \
                     conflicts with integration after step 2 in:",
                    "Cargo.lock",
                    NOTHING_REACHED,
                ],
                &["Cargo.lock"],
                3,
                C_COMMIT,
            ),
        ),
    ];
    for (name, with, answer, expected) in rows {
        assert_eq!(would_merge_with_report(&squash(A_COMMIT, A_ID), &with, &answer), expected, "row {name}");
    }
}
