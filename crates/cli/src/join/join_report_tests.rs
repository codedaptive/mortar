//! join_report: each join answer as its exact report, after the save's lines and fields.
//! Unit tests: join/save_part is pure and used directly.

use std::path::PathBuf;

use ::join::JoinAnswer;
use gitio::Oid;
use snapshot::SaveOutcome;

use super::{join_report, JoinOutcome};
use crate::report::{ExitCode, OutValue, Report};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const ZONE: &str = "refs/heads/zone/core";
const HEAD: &str = "4444444444444444444444444444444444444444";
const TREE: &str = "5555555555555555555555555555555555555555";
const FROM: &str = "1111111111111111111111111111111111111111";
const TO: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn outcome(answer: JoinAnswer) -> JoinOutcome {
    JoinOutcome {
        request_ref: REQUEST.to_string(),
        snapshot_written: false,
        finished_move: None,
        save: SaveOutcome { commit: oid(HEAD), tree: oid(TREE), written: false, change_id: None, refused: Vec::new() },
        answer,
    }
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

/// request_ref, then save_part's fields for the unwritten save above, then `answer`'s.
fn fields(answer: Vec<(String, OutValue)>) -> Vec<(String, OutValue)> {
    let mut all = vec![
        text("request_ref", REQUEST),
        ("finished_move_from".to_string(), OutValue::Null),
        ("finished_move_to".to_string(), OutValue::Null),
        text("saved_commit", HEAD),
        ("saved_written".to_string(), OutValue::Bool(false)),
        ("change_id".to_string(), OutValue::Null),
        ("refused".to_string(), OutValue::Texts(Vec::new())),
    ];
    all.extend(answer);
    all
}

#[test]
fn joined_waits_for_the_zones_tests() {
    let answer = JoinAnswer::Joined { zone_ref: ZONE.to_string(), request_tip: oid(TO) };
    assert_eq!(
        join_report(&outcome(answer)),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Done,
            outcome: "joined".to_string(),
            lines: vec![
                "no unsaved work".to_string(),
                "joined: the request tip 2222222222222222222222222222222222222222 is already in \
                 refs/heads/zone/core; nothing moved"
                    .to_string(),
            ],
            owner: None,
            next: "wait for the zone's tests".to_string(),
            fields: fields(vec![text("zone_ref", ZONE), text("request_tip", TO)]),
        }
    );
}

#[test]
fn fast_forwarded_is_joined() {
    let answer = JoinAnswer::FastForwarded { zone_ref: ZONE.to_string(), from: oid(FROM), to: oid(TO) };
    assert_eq!(
        join_report(&outcome(answer)),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Done,
            outcome: "joined".to_string(),
            lines: vec![
                "no unsaved work".to_string(),
                "joined: refs/heads/zone/core moved forward from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222"
                    .to_string(),
            ],
            owner: None,
            next: "wait for the zone's tests".to_string(),
            fields: fields(vec![text("zone_ref", ZONE), text("from", FROM), text("to", TO)]),
        }
    );
}

#[test]
fn rebased_is_done_and_joins_again() {
    let answer =
        JoinAnswer::Rebased { request_ref: REQUEST.to_string(), from: oid(FROM), to: oid(TO), folder_tree: None };
    assert_eq!(
        join_report(&outcome(answer)),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Done,
            outcome: "rebased".to_string(),
            lines: vec![
                "no unsaved work".to_string(),
                "rebased: refs/heads/req/core/fix-1 moved from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222, replayed onto the zone; the folder moved with it"
                    .to_string(),
                "the unit tests must run again".to_string(),
            ],
            owner: None,
            next: "mr join".to_string(),
            fields: fields(vec![text("from", FROM), text("to", TO)]),
        }
    );
}

#[test]
fn conflicted_is_exit_2_listing_the_files() {
    let answer = JoinAnswer::Conflicted {
        commit: oid(FROM),
        files: vec![PathBuf::from("src/a.rs"), PathBuf::from("docs/b.md")],
        state: PathBuf::from("/r/.git/worktrees/fix-1/mortar-join.json"),
    };
    assert_eq!(
        join_report(&outcome(answer)),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflicted".to_string(),
            lines: vec![
                "no unsaved work".to_string(),
                "conflicted: commit 1111111111111111111111111111111111111111 did not apply; no ref moved and the \
                 folder stays at the request tip"
                    .to_string(),
                "  src/a.rs".to_string(),
                "  docs/b.md".to_string(),
                "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again"
                    .to_string(),
            ],
            owner: None,
            next: "mr resolve".to_string(),
            fields: fields(vec![
                text("commit", FROM),
                ("files".to_string(), OutValue::Texts(vec!["src/a.rs".to_string(), "docs/b.md".to_string()])),
                text("state", "/r/.git/worktrees/fix-1/mortar-join.json"),
            ]),
        }
    );
}

#[test]
fn conflicted_names_both_refs_moved_by_save_first() {
    let answer = JoinAnswer::Conflicted {
        commit: oid(FROM), files: vec![PathBuf::from("src/a.rs")],
        state: PathBuf::from("/r/.git/worktrees/fix-1/mortar-join.json"),
    };
    let mut result = outcome(answer);
    result.save.written = true;
    result.snapshot_written = true;
    assert_eq!(join_report(&result), Report {
        verb: "join".to_string(), exit: ExitCode::Conflict, outcome: "conflicted".to_string(),
        lines: vec![
            "work saved at join as 4444444444444444444444444444444444444444".to_string(),
            "conflicted: commit 1111111111111111111111111111111111111111 did not apply; refs/heads/req/core/fix-1 and the snapshot ref moved as the work was saved first; no other ref moved and the folder stays at the request tip".to_string(),
            "  src/a.rs".to_string(),
            "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again".to_string(),
        ],
        owner: None, next: "mr resolve".to_string(),
        fields: vec![
            ("request_ref".to_string(), OutValue::Text("refs/heads/req/core/fix-1".to_string())),
            ("finished_move_from".to_string(), OutValue::Null), ("finished_move_to".to_string(), OutValue::Null),
            ("saved_commit".to_string(), OutValue::Text("4444444444444444444444444444444444444444".to_string())),
            ("saved_written".to_string(), OutValue::Bool(true)), ("change_id".to_string(), OutValue::Null),
            ("refused".to_string(), OutValue::Texts(vec![])),
            ("commit".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
            ("files".to_string(), OutValue::Texts(vec!["src/a.rs".to_string()])),
            ("state".to_string(), OutValue::Text("/r/.git/worktrees/fix-1/mortar-join.json".to_string())),
        ],
    });
}

#[test]
fn joined_names_a_snapshot_only_move() {
    let answer = JoinAnswer::Joined { zone_ref: ZONE.to_string(), request_tip: oid(TO) };
    let mut result = outcome(answer);
    result.snapshot_written = true;
    assert_eq!(join_report(&result), Report {
        verb: "join".to_string(), exit: ExitCode::Done, outcome: "joined".to_string(),
        lines: vec![
            "no unsaved work".to_string(),
            "joined: the request tip 2222222222222222222222222222222222222222 is already in refs/heads/zone/core; the snapshot ref moved as the work was saved first; nothing else moved".to_string(),
        ],
        owner: None, next: "wait for the zone's tests".to_string(),
        fields: vec![
            ("request_ref".to_string(), OutValue::Text("refs/heads/req/core/fix-1".to_string())),
            ("finished_move_from".to_string(), OutValue::Null), ("finished_move_to".to_string(), OutValue::Null),
            ("saved_commit".to_string(), OutValue::Text("4444444444444444444444444444444444444444".to_string())),
            ("saved_written".to_string(), OutValue::Bool(false)), ("change_id".to_string(), OutValue::Null),
            ("refused".to_string(), OutValue::Texts(vec![])),
            ("zone_ref".to_string(), OutValue::Text("refs/heads/zone/core".to_string())),
            ("request_tip".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
        ],
    });
}

#[test]
fn files_written_is_a_failure() {
    let answer =
        JoinAnswer::FilesWritten { commit: oid(FROM), files: Vec::new(), skipped: Vec::new(), gitlinks: Vec::new() };
    assert_eq!(
        join_report(&outcome(answer)),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "join answered files-written, which only resolve gives".to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
