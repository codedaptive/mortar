//! base_answer_part: each answer of a join on a recorded base as its exact lines, fields, exit, outcome and next;
//! a rebase with no partners left is the plain rebase's; a conflicting new merge and files-written give none.

use std::path::PathBuf;

use ::join::{BaseJoinAnswer, JoinAnswer, Partner, RequestBase};
use gitio::Oid;

use super::{base_answer_part, AnswerPart};
use crate::report::{ExitCode, OutValue};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const ZONE: &str = "refs/heads/zone/core";
const ONE: &str = "1111111111111111111111111111111111111111";
const TWO: &str = "2222222222222222222222222222222222222222";
const THREE: &str = "3333333333333333333333333333333333333333";
const MERGE: &str = "4444444444444444444444444444444444444444";
const WEB: &str = "5555555555555555555555555555555555555555";
const MOVED: &str = "refs/heads/req/core/fix-1 moved as the work was saved first";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn zone_joined() -> BaseJoinAnswer {
    BaseJoinAnswer::ZoneJoined {
        zone_ref: ZONE.to_string(),
        request_ref: REQUEST.to_string(),
        from: oid(ONE),
        request_from: oid(TWO),
        to: oid(THREE),
    }
}

fn rebased() -> BaseJoinAnswer {
    BaseJoinAnswer::Rebased {
        request_ref: REQUEST.to_string(),
        from: oid(TWO),
        to: oid(THREE),
        base: RequestBase { base: oid(MERGE), partners: vec![Partner { zone: "web".to_string(), tip: oid(WEB) }] },
    }
}

fn joined_part(line: &str) -> AnswerPart {
    AnswerPart {
        lines: vec![line.to_string()],
        fields: vec![text("zone_ref", ZONE), text("from", ONE), text("to", THREE), text("request_from", TWO)],
        exit: ExitCode::Done,
        outcome: "joined",
        next: "wait for the zone's tests",
    }
}

fn rebased_part(line: &str) -> AnswerPart {
    AnswerPart {
        lines: vec![line.to_string(), "the unit tests must run again on the new merge".to_string()],
        fields: vec![
            text("from", TWO),
            text("to", THREE),
            text("base", MERGE),
            (
                "partners".to_string(),
                OutValue::Raw(vec![r#"{"zone":"web","tip":"5555555555555555555555555555555555555555"}"#.to_string()]),
            ),
        ],
        exit: ExitCode::Done,
        outcome: "rebased",
        next: "run the unit tests, then mr join",
    }
}

#[test]
fn base_answer_part_cases() {
    let cases: Vec<(&str, BaseJoinAnswer, Option<&str>, Option<AnswerPart>)> = vec![
        (
            "plain joined is answer_part's",
            BaseJoinAnswer::Plain(JoinAnswer::Joined { zone_ref: ZONE.to_string(), request_tip: oid(THREE) }),
            None,
            Some(AnswerPart {
                lines: vec![
                    "joined: the request tip 3333333333333333333333333333333333333333 is already in \
                     refs/heads/zone/core; nothing moved"
                        .to_string(),
                ],
                fields: vec![text("zone_ref", ZONE), text("request_tip", THREE)],
                exit: ExitCode::Done,
                outcome: "joined",
                next: "wait for the zone's tests",
            }),
        ),
        (
            "zone joined names both moves",
            zone_joined(),
            None,
            Some(joined_part(
                "joined: refs/heads/zone/core moved from 1111111111111111111111111111111111111111 to \
                 3333333333333333333333333333333333333333, taking only the request's own commits; \
                 refs/heads/req/core/fix-1 moved from 2222222222222222222222222222222222222222 to \
                 3333333333333333333333333333333333333333 and the folder moved with it",
            )),
        ),
        (
            "zone joined after a save names it last",
            zone_joined(),
            Some(MOVED),
            Some(joined_part(
                "joined: refs/heads/zone/core moved from 1111111111111111111111111111111111111111 to \
                 3333333333333333333333333333333333333333, taking only the request's own commits; \
                 refs/heads/req/core/fix-1 moved from 2222222222222222222222222222222222222222 to \
                 3333333333333333333333333333333333333333 and the folder moved with it; \
                 refs/heads/req/core/fix-1 moved as the work was saved first",
            )),
        ),
        (
            "rebased onto a new merge stops for the unit tests",
            rebased(),
            None,
            Some(rebased_part(
                "rebased: refs/heads/req/core/fix-1 moved from 2222222222222222222222222222222222222222 to \
                 3333333333333333333333333333333333333333, replayed onto a new merge \
                 4444444444444444444444444444444444444444 of the zone with its partners; the folder moved with it",
            )),
        ),
        (
            "rebased after a save names it last",
            rebased(),
            Some(MOVED),
            Some(rebased_part(
                "rebased: refs/heads/req/core/fix-1 moved from 2222222222222222222222222222222222222222 to \
                 3333333333333333333333333333333333333333, replayed onto a new merge \
                 4444444444444444444444444444444444444444 of the zone with its partners; the folder moved with it; \
                 refs/heads/req/core/fix-1 moved as the work was saved first",
            )),
        ),
        (
            "rebased with no partners left is the plain rebase",
            BaseJoinAnswer::Rebased {
                request_ref: REQUEST.to_string(),
                from: oid(TWO),
                to: oid(THREE),
                base: RequestBase { base: oid(MERGE), partners: Vec::new() },
            },
            Some(MOVED),
            Some(AnswerPart {
                lines: vec![
                    "rebased: refs/heads/req/core/fix-1 moved from 2222222222222222222222222222222222222222 to \
                     3333333333333333333333333333333333333333, replayed onto the zone; the folder moved with it; \
                     refs/heads/req/core/fix-1 moved as the work was saved first"
                        .to_string(),
                    "the unit tests must run again".to_string(),
                ],
                fields: vec![text("from", TWO), text("to", THREE)],
                exit: ExitCode::Done,
                outcome: "rebased",
                next: "mr join",
            }),
        ),
        (
            "partners conflicted gives none",
            BaseJoinAnswer::PartnersConflicted { zone: "web".to_string(), files: vec![PathBuf::from("Cargo.toml")] },
            None,
            None,
        ),
        (
            "files written gives none",
            BaseJoinAnswer::Plain(JoinAnswer::FilesWritten {
                commit: oid(ONE),
                files: Vec::new(),
                skipped: Vec::new(),
                gitlinks: Vec::new(),
            }),
            None,
            None,
        ),
    ];
    for (name, answer, moved, expected) in cases {
        assert_eq!(base_answer_part(&answer, REQUEST, moved), expected, "case {name}");
    }
}
