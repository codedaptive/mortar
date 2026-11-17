//! choose_undo over histories given as data, where the chosen op is a join-rebase recording the paused join (a resolve
//! step, S-148) or an undo of a join-rebase: a resolve step restores the paused join, and an undo of one is already
//! undone with that step; an undo of a join-rebase missing from the log is already undone with the folder step.
//! ZIT (S-102, rule 11): choose_undo decides these through undo_reason's is_resolve_step and done_record, run for real.

use ::join::JOIN_STATE;
use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove, ValuedRecord};

use cli::undo::{choose_undo, UndoChoice, UndoStep};

const ACTOR: &str = "orc/WO-cli-4";
const ZONE: &str = "refs/heads/zone/orc";
const REQ: &str = "refs/heads/req/orc/WO-cli-4";
const SNAP: &str = "refs/mortar/snap/orc/WO-cli-4";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn op(op_id: &str, kind: &str, actor: &str, phase: OpPhase, refs: &[&str]) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("key-{op_id}"),
            kind: kind.to_string(),
            actor: actor.to_string(),
            reason: "a step".to_string(),
            at: 100,
            phase,
            moves: refs
                .iter()
                .map(|refname| RefMove { refname: refname.to_string(), old: Some(oid('b')), new: oid('c') })
                .collect(),
            result: (phase == OpPhase::Done).then_some(OpResult::Moved),
        },
        values: Vec::new(),
    }
}

fn done(op_id: &str, kind: &str, refs: &[&str]) -> ValuedRecord {
    op(op_id, kind, ACTOR, OpPhase::Done, refs)
}

fn undo_op(op_id: &str, reason: &str) -> ValuedRecord {
    let mut record = done(op_id, "undo", &[REQ]);
    record.record.reason = reason.to_string();
    record
}

fn request_own() -> Vec<String> {
    vec![REQ.to_string(), SNAP.to_string()]
}

fn choose(records: &[ValuedRecord]) -> UndoChoice {
    choose_undo(records, ACTOR, &request_own(), ZONE)
}

#[test]
fn a_join_rebase_with_a_join_state_restores_the_paused_join() {
    let mut resolve = done("op-2", "join-rebase", &[REQ]);
    resolve.values = vec![(JOIN_STATE.to_string(), "{}".to_string())];
    let records = [done("op-1", "save", &[REQ]), resolve];
    assert_eq!(choose(&records), UndoChoice::Step { index: 1, step: UndoStep::RestoreJoin });
}

#[test]
fn a_join_rebase_with_a_join_state_and_a_folder_tree_restores_the_paused_join() {
    let mut resolve = done("op-2", "join-rebase", &[REQ]);
    resolve.values = vec![
        ("folder_tree".to_string(), "4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string()),
        (JOIN_STATE.to_string(), "{}".to_string()),
    ];
    let records = [done("op-1", "save", &[REQ]), resolve];
    assert_eq!(choose(&records), UndoChoice::Step { index: 1, step: UndoStep::RestoreJoin });
}

#[test]
fn an_undo_of_a_resolve_step_is_already_undone_with_the_restore_step() {
    let mut resolve = done("op-1", "join-rebase", &[REQ]);
    resolve.values = vec![(JOIN_STATE.to_string(), "{}".to_string())];
    let records = [resolve, undo_op("op-2", "undo of join-rebase op op-1")];
    assert_eq!(
        choose(&records),
        UndoChoice::AlreadyUndone {
            undo_op: "op-2".to_string(),
            reversed_op: "op-1".to_string(),
            step: UndoStep::RestoreJoin,
        }
    );
}

#[test]
fn an_undo_of_a_join_rebase_missing_from_the_log_is_already_undone_with_the_folder_step() {
    let records = [done("op-1", "save", &[REQ]), undo_op("op-2", "undo of join-rebase op op-9")];
    assert_eq!(
        choose(&records),
        UndoChoice::AlreadyUndone {
            undo_op: "op-2".to_string(),
            reversed_op: "op-9".to_string(),
            step: UndoStep::MoveFolder,
        }
    );
}
