//! undo_refusal: a completed join, a resolve and an op that is not a step of this folder are refused (exit 1) with
//! nothing changed and a next in words an agent can follow; nothing, a step and an already-undone op are not refused
//! (S-73; S-142 item 1).

use super::{undo_refusal, UndoChoice, UndoStep};
use crate::report::{ExitCode, Report};

const NOTHING_CHANGED: &str = "nothing was changed";
/// Commands an agent cannot run: harness verbs, and --actor, which agent verbs refuse.
const UNREACHABLE: [&str; 8] = [
    "mortar restore",
    "mortar back-out",
    "mortar folder",
    "mortar snapshot",
    "mortar land",
    "mortar publish",
    "mortar after-publish",
    "--actor",
];

fn refused(lines: &[&str], owner: Option<&str>, next: &str) -> Option<Report> {
    Some(Report {
        verb: "undo".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: owner.map(str::to_string),
        next: next.to_string(),
        fields: Vec::new(),
    })
}

fn zone_owns() -> UndoChoice {
    UndoChoice::ZoneOwns { op_id: "op-9".to_string() }
}

fn resolved() -> UndoChoice {
    UndoChoice::Resolved { op_id: "op-8".to_string() }
}

fn not_a_step() -> UndoChoice {
    UndoChoice::NotAStep { op_id: "op-7".to_string(), kind: "folder-move".to_string() }
}

#[test]
fn a_completed_join_is_refused_to_the_zones_coordinator() {
    assert_eq!(
        undo_refusal(&zone_owns()),
        refused(
            &["op op-9 is a completed join: the zone owns it now", NOTHING_CHANGED],
            Some("the zone's coordinator"),
            "ask the zone's coordinator to back out op op-9",
        )
    );
}

#[test]
fn a_resolve_is_refused_with_the_way_forward_in_words() {
    assert_eq!(
        undo_refusal(&resolved()),
        refused(
            &["op op-8 resolved a join, and undo cannot put the paused join back", NOTHING_CHANGED],
            None,
            "change the files, mortar save, then mortar join",
        )
    );
}

#[test]
fn an_op_that_is_not_a_step_is_refused_naming_it_and_its_kind() {
    assert_eq!(
        undo_refusal(&not_a_step()),
        refused(
            &[
                "op op-7 is a folder-move: only a save, a rebase or a resolve of this folder can be undone",
                NOTHING_CHANGED,
            ],
            None,
            "mortar status",
        )
    );
}

#[test]
fn no_refusal_names_a_command_an_agent_cannot_run() {
    for choice in [zone_owns(), resolved(), not_a_step()] {
        let report = undo_refusal(&choice).expect("a refusal");
        for command in UNREACHABLE {
            assert!(!report.next.contains(command), "{report:?}");
            assert!(!report.lines.iter().any(|line| line.contains(command)), "{report:?}");
        }
    }
}

#[test]
fn nothing_a_step_and_an_already_undone_op_are_not_refused() {
    for choice in [
        UndoChoice::Nothing,
        UndoChoice::Step { index: 3, step: UndoStep::SetIndex },
        UndoChoice::Step { index: 0, step: UndoStep::MoveFolder },
        UndoChoice::AlreadyUndone {
            undo_op: "op-2".to_string(),
            reversed_op: "op-1".to_string(),
            step: UndoStep::SetIndex,
        },
    ] {
        assert_eq!(undo_refusal(&choice), None, "{choice:?}");
    }
}
