//! answer_part: each join answer's exact lines, fields, exit, outcome and next; files-written gives none.

use std::path::PathBuf;

use ::join::JoinAnswer;
use gitio::Oid;

use super::{answer_part, AnswerPart};
use crate::report::{ExitCode, OutValue};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const ZONE: &str = "refs/heads/zone/core";
const FROM: &str = "1111111111111111111111111111111111111111";
const TO: &str = "2222222222222222222222222222222222222222";
const COMMIT: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

#[test]
fn joined_names_the_tip_already_in_the_zone() {
    let answer = JoinAnswer::Joined { zone_ref: ZONE.to_string(), request_tip: oid(TO) };
    assert_eq!(
        answer_part(&answer, REQUEST, None),
        Some(AnswerPart {
            lines: vec![
                "joined: the request tip 2222222222222222222222222222222222222222 is already in \
                 refs/heads/zone/core; nothing moved"
                    .to_string()
            ],
            fields: vec![text("zone_ref", ZONE), text("request_tip", TO)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: "wait for the zone's tests",
        })
    );
}

#[test]
fn fast_forwarded_is_joined_naming_both_tips() {
    let answer = JoinAnswer::FastForwarded { zone_ref: ZONE.to_string(), from: oid(FROM), to: oid(TO) };
    assert_eq!(
        answer_part(&answer, REQUEST, None),
        Some(AnswerPart {
            lines: vec![
                "joined: refs/heads/zone/core moved forward from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222"
                    .to_string()
            ],
            fields: vec![text("zone_ref", ZONE), text("from", FROM), text("to", TO)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: "wait for the zone's tests",
        })
    );
}

#[test]
fn fast_forwarded_names_the_move_that_preceded_it() {
    let answer = JoinAnswer::FastForwarded { zone_ref: ZONE.to_string(), from: oid(FROM), to: oid(TO) };
    assert_eq!(
        answer_part(&answer, REQUEST, Some("the snapshot ref moved as the work was saved first")),
        Some(AnswerPart {
            lines: vec![
                "joined: refs/heads/zone/core moved forward from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222; the snapshot ref moved as the work was saved first"
                    .to_string()
            ],
            fields: vec![text("zone_ref", ZONE), text("from", FROM), text("to", TO)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: "wait for the zone's tests",
        })
    );
}

#[test]
fn rebased_says_the_tests_run_again_and_joins_next() {
    let answer =
        JoinAnswer::Rebased { request_ref: REQUEST.to_string(), from: oid(FROM), to: oid(TO), folder_tree: None };
    assert_eq!(
        answer_part(&answer, REQUEST, None),
        Some(AnswerPart {
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222, replayed onto the zone; the folder moved with it"
                    .to_string(),
                "the unit tests must run again".to_string(),
            ],
            fields: vec![text("from", FROM), text("to", TO)],
            exit: ExitCode::Done,
            outcome: "rebased",
            next: "mr join",
        })
    );
}

#[test]
fn rebased_names_the_move_that_preceded_it() {
    let answer =
        JoinAnswer::Rebased { request_ref: REQUEST.to_string(), from: oid(FROM), to: oid(TO), folder_tree: None };
    assert_eq!(
        answer_part(&answer, REQUEST, Some("the snapshot ref moved as the work was saved first")),
        Some(AnswerPart {
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222, replayed onto the zone; the folder moved with it; \
                 the snapshot ref moved as the work was saved first".to_string(),
                "the unit tests must run again".to_string(),
            ],
            fields: vec![text("from", FROM), text("to", TO)],
            exit: ExitCode::Done,
            outcome: "rebased",
            next: "mr join",
        })
    );
}

#[test]
fn conflicted_lists_each_file_and_resolves_next() {
    let answer = JoinAnswer::Conflicted {
        commit: oid(COMMIT),
        files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        state: PathBuf::from("/w/.git/mortar-join.json"),
    };
    assert_eq!(
        answer_part(&answer, REQUEST, None),
        Some(AnswerPart {
            lines: vec![
                "conflicted: commit 3333333333333333333333333333333333333333 did not apply; no ref moved and the \
                 folder stays at the request tip"
                    .to_string(),
                "  src/a.rs".to_string(),
                "  b.txt".to_string(),
                "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again"
                    .to_string(),
            ],
            fields: vec![
                text("commit", COMMIT),
                ("files".to_string(), OutValue::Texts(vec!["src/a.rs".to_string(), "b.txt".to_string()])),
                text("state", "/w/.git/mortar-join.json"),
            ],
            exit: ExitCode::Conflict,
            outcome: "conflicted",
            next: "mr resolve",
        })
    );
}

#[test]
fn joined_names_the_move_that_preceded_it() {
    let answer = JoinAnswer::Joined { zone_ref: ZONE.to_string(), request_tip: oid(TO) };
    assert_eq!(
        answer_part(&answer, REQUEST, Some("the snapshot ref moved as the work was saved first")),
        Some(AnswerPart {
            lines: vec!["joined: the request tip 2222222222222222222222222222222222222222 is already in refs/heads/zone/core; the snapshot ref moved as the work was saved first; nothing else moved".to_string()],
            fields: vec![("zone_ref".to_string(), OutValue::Text("refs/heads/zone/core".to_string())), ("request_tip".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string()))],
            exit: ExitCode::Done,
            outcome: "joined",
            next: "wait for the zone's tests",
        })
    );
}

#[test]
fn conflicted_names_the_move_that_preceded_it() {
    let answer = JoinAnswer::Conflicted {
        commit: oid(COMMIT), files: vec![PathBuf::from("src/a.rs")],
        state: PathBuf::from("/w/.git/mortar-join.json"),
    };
    assert_eq!(
        answer_part(&answer, REQUEST, Some("refs/heads/req/core/fix-1 moved as the work was saved first")),
        Some(AnswerPart {
            lines: vec![
                "conflicted: commit 3333333333333333333333333333333333333333 did not apply; refs/heads/req/core/fix-1 moved as the work was saved first; no other ref moved and the folder stays at the request tip".to_string(),
                "  src/a.rs".to_string(),
                "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again".to_string(),
            ],
            fields: vec![
                ("commit".to_string(), OutValue::Text("3333333333333333333333333333333333333333".to_string())),
                ("files".to_string(), OutValue::Texts(vec!["src/a.rs".to_string()])),
                ("state".to_string(), OutValue::Text("/w/.git/mortar-join.json".to_string())),
            ],
            exit: ExitCode::Conflict, outcome: "conflicted", next: "mr resolve",
        })
    );
}

#[test]
fn files_written_gives_no_part() {
    let answer = JoinAnswer::FilesWritten {
        commit: oid(COMMIT),
        files: vec![PathBuf::from("src/a.rs")],
        skipped: Vec::new(),
        gitlinks: Vec::new(),
    };
    assert_eq!(answer_part(&answer, REQUEST, None), None);
}
