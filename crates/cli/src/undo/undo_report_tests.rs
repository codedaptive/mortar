//! undo_report: a done undo is exit 0 "undone", naming the op undone, the undo op and the snapshot commit kept of the
//! folder as it stood; undoing a save leaves the files untouched and shows the work as unsaved; undoing a resolve
//! leaves the files untouched, puts the branch back at the tip before the resolve and sends the agent to mr resolve.
//! The --json fields are not pinned here.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

use super::{undo_report, UndoStep};
use crate::report::{ExitCode, Report};

const SNAPSHOT: &str = "0123456789abcdef0123456789abcdef01234567";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn save_op() -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "save-key".to_string(),
        kind: "save".to_string(),
        actor: "orc/WO-cli-4".to_string(),
        reason: "fix the parser".to_string(),
        at: 40,
        phase: OpPhase::Done,
        moves: vec![RefMove {
            refname: "refs/heads/req/orc/WO-cli-4".to_string(),
            old: Some(oid(&"b".repeat(40))),
            new: oid(&"c".repeat(40)),
        }],
        result: Some(OpResult::Moved),
    }
}

/// The report with its --json fields left out.
fn shown(report: Report) -> Report {
    Report { fields: Vec::new(), ..report }
}

#[test]
fn an_undone_save_names_the_op_the_undo_and_the_snapshot_and_shows_the_work_unsaved() {
    assert_eq!(
        shown(undo_report(&save_op(), "op-12", &oid(SNAPSHOT), UndoStep::SetIndex)),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Done,
            outcome: "undone".to_string(),
            lines: vec![
                "undid save op op-7".to_string(),
                "the undo is op op-12".to_string(),
                "the folder as it stood is kept in snapshot commit 0123456789abcdef0123456789abcdef01234567"
                    .to_string(),
                "the files are untouched; the work shows as unsaved".to_string(),
            ],
            owner: None,
            next: "mr status".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn an_undone_rebase_says_the_folder_is_moved_back_to_the_tip_before_it() {
    let rebase = OpRecord { op_id: "op-8".to_string(), kind: "join-rebase".to_string(), ..save_op() };
    let report = undo_report(&rebase, "op-13", &oid(SNAPSHOT), UndoStep::MoveFolder);
    assert_eq!(report.outcome, "undone");
    assert_eq!(report.lines[0], "undid join-rebase op op-8");
    assert_eq!(report.lines[3], "the folder is moved back to the tip before the rebase");
    assert_eq!(report.next, "mr status");
}

#[test]
fn an_undone_resolve_keeps_the_files_and_sends_the_agent_to_resolve_again() {
    let resolve = OpRecord { op_id: "op-9".to_string(), kind: "join-rebase".to_string(), ..save_op() };
    assert_eq!(
        shown(undo_report(&resolve, "op-14", &oid(SNAPSHOT), UndoStep::RestoreJoin)),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Done,
            outcome: "undone".to_string(),
            lines: vec![
                "undid join-rebase op op-9".to_string(),
                "the undo is op op-14".to_string(),
                "the folder as it stood is kept in snapshot commit 0123456789abcdef0123456789abcdef01234567"
                    .to_string(),
                "the files are untouched; the branch is back at the tip before the resolve".to_string(),
            ],
            owner: None,
            next: "mr resolve".to_string(),
            fields: Vec::new(),
        }
    );
}
