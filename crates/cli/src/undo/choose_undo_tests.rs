//! choose_undo over histories given as data: the last done op of the folder's actor, kind not "snapshot", that moved
//! any of the folder's own refs or the zone branch, and what undo does with it (S-73; S-142 items 1-3). A join-rebase
//! recording the paused join (a resolve step, S-148) restores it, and an undo of one is already undone with that
//! step; an undo of a join-rebase missing from the log is already undone with the folder step.
//! ZUT (rule 11): undo_reason's is_resolve_step and done_record are pure and used directly.

use ::join::JOIN_STATE;
use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove, ValuedRecord};

use super::{choose_undo, UndoChoice, UndoStep};

const ACTOR: &str = "orc/WO-cli-4";
const ZONE: &str = "refs/heads/zone/orc";
const REQ: &str = "refs/heads/req/orc/WO-cli-4";
const SNAP: &str = "refs/mortar/snap/orc/WO-cli-4";
const TRY2: &str = "refs/heads/try/orc/WO-cli-4/2";
const TRYSNAP2: &str = "refs/mortar/trysnap/orc/WO-cli-4/2";

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

fn attempt_own() -> Vec<String> {
    vec![TRY2.to_string(), TRYSNAP2.to_string()]
}

fn choose(records: &[ValuedRecord]) -> UndoChoice {
    choose_undo(records, ACTOR, &request_own(), ZONE)
}

#[test]
fn an_empty_log_is_nothing() {
    assert_eq!(choose(&[]), UndoChoice::Nothing);
}

#[test]
fn a_log_of_only_snapshots_is_nothing() {
    let records = [done("op-1", "snapshot", &[SNAP]), done("op-2", "snapshot", &[SNAP])];
    assert_eq!(choose(&records), UndoChoice::Nothing);
}

#[test]
fn a_save_is_chosen_past_a_later_end_of_turn_snapshot_of_the_same_actor() {
    let records = [done("op-1", "snapshot", &[SNAP]), done("op-2", "save", &[REQ]), done("op-3", "snapshot", &[SNAP])];
    assert_eq!(choose(&records), UndoChoice::Step { index: 1, step: UndoStep::SetIndex });
}

#[test]
fn a_later_op_by_another_actor_on_the_same_ref_is_not_chosen() {
    let records = [
        done("op-1", "save", &[REQ]),
        done("op-2", "save", &[REQ]),
        op("op-3", "save", "orc/someone-else", OpPhase::Done, &[REQ]),
    ];
    assert_eq!(choose(&records), UndoChoice::Step { index: 1, step: UndoStep::SetIndex });
}

#[test]
fn intent_and_failed_records_are_skipped() {
    let records = [
        op("op-1", "save", ACTOR, OpPhase::Intent, &[REQ]),
        done("op-1", "save", &[REQ]),
        op("op-2", "save", ACTOR, OpPhase::Intent, &[REQ]),
        op("op-2", "save", ACTOR, OpPhase::Failed, &[REQ]),
        op("op-3", "join-rebase", ACTOR, OpPhase::Intent, &[REQ]),
    ];
    assert_eq!(choose(&records), UndoChoice::Step { index: 1, step: UndoStep::SetIndex });
}

#[test]
fn ops_on_refs_neither_own_nor_the_zones_are_not_reached() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "save", &["refs/heads/req/orc/WO-cli-9"])];
    assert_eq!(choose(&records), UndoChoice::Step { index: 0, step: UndoStep::SetIndex });
}

#[test]
fn a_completed_join_is_owned_by_the_zone() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "join-fast-forward", &[ZONE])];
    assert_eq!(choose(&records), UndoChoice::ZoneOwns { op_id: "op-2".to_string() });
}

#[test]
fn a_join_rebase_without_a_folder_tree_moves_the_folder() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "join-rebase", &[REQ])];
    assert_eq!(choose(&records), UndoChoice::Step { index: 1, step: UndoStep::MoveFolder });
}

#[test]
fn a_join_rebase_with_a_folder_tree_is_a_resolve() {
    let mut resolve = done("op-2", "join-rebase", &[REQ]);
    resolve.values = vec![("folder_tree".to_string(), "4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string())];
    let records = [done("op-1", "save", &[REQ]), resolve];
    assert_eq!(choose(&records), UndoChoice::Resolved { op_id: "op-2".to_string() });
}

#[test]
fn a_join_rebase_with_other_values_only_still_moves_the_folder() {
    let mut rebase = done("op-2", "join-rebase", &[REQ]);
    rebase.values = vec![("onto".to_string(), "x".to_string())];
    assert_eq!(choose(&[rebase]), UndoChoice::Step { index: 0, step: UndoStep::MoveFolder });
}

#[test]
fn an_undo_of_a_save_is_already_undone_with_the_index_step() {
    let records = [done("op-1", "save", &[REQ]), undo_op("op-2", "undo of save op op-1")];
    assert_eq!(
        choose(&records),
        UndoChoice::AlreadyUndone {
            undo_op: "op-2".to_string(),
            reversed_op: "op-1".to_string(),
            step: UndoStep::SetIndex,
        }
    );
}

#[test]
fn an_undo_of_a_join_rebase_is_already_undone_with_the_folder_step() {
    let records = [done("op-1", "join-rebase", &[REQ]), undo_op("op-2", "undo of join-rebase op op-1")];
    assert_eq!(
        choose(&records),
        UndoChoice::AlreadyUndone {
            undo_op: "op-2".to_string(),
            reversed_op: "op-1".to_string(),
            step: UndoStep::MoveFolder,
        }
    );
}

#[test]
fn an_undo_whose_reason_cannot_be_read_is_not_a_step() {
    let records = [done("op-1", "save", &[REQ]), undo_op("op-2", "put it back")];
    assert_eq!(choose(&records), UndoChoice::NotAStep { op_id: "op-2".to_string(), kind: "undo".to_string() });
}

#[test]
fn a_folder_move_is_not_a_step() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "folder-move", &[REQ])];
    assert_eq!(
        choose(&records),
        UndoChoice::NotAStep { op_id: "op-2".to_string(), kind: "folder-move".to_string() }
    );
}

#[test]
fn a_restore_is_not_a_step() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "restore", &[REQ, SNAP])];
    assert_eq!(choose(&records), UndoChoice::NotAStep { op_id: "op-2".to_string(), kind: "restore".to_string() });
}

#[test]
fn a_save_that_also_moved_a_ref_not_its_own_is_not_a_step() {
    let records = [done("op-1", "save", &[REQ, "refs/heads/req/orc/WO-cli-9"])];
    assert_eq!(choose(&records), UndoChoice::NotAStep { op_id: "op-1".to_string(), kind: "save".to_string() });
}

#[test]
fn an_attempts_save_is_not_reached_from_the_request_folder() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "save", &[TRY2]), done("op-3", "snapshot", &[TRYSNAP2])];
    assert_eq!(choose(&records), UndoChoice::Step { index: 0, step: UndoStep::SetIndex });
}

#[test]
fn the_requests_save_is_not_reached_from_attempt_two() {
    let records = [done("op-1", "save", &[TRY2]), done("op-2", "save", &[REQ]), done("op-3", "snapshot", &[SNAP])];
    assert_eq!(
        choose_undo(&records, ACTOR, &attempt_own(), ZONE),
        UndoChoice::Step { index: 0, step: UndoStep::SetIndex }
    );
}

#[test]
fn only_the_requests_ops_from_attempt_two_is_nothing() {
    let records = [done("op-1", "save", &[REQ]), done("op-2", "join-rebase", &[REQ])];
    assert_eq!(choose_undo(&records, ACTOR, &attempt_own(), ZONE), UndoChoice::Nothing);
}

#[test]
fn step_carries_the_chosen_records_index_in_the_slice() {
    let records = [
        done("op-1", "snapshot", &[SNAP]),
        op("op-2", "save", ACTOR, OpPhase::Intent, &[REQ]),
        done("op-2", "save", &[REQ]),
        done("op-3", "snapshot", &[SNAP]),
        op("op-4", "save", "orc/someone-else", OpPhase::Done, &[REQ]),
    ];
    assert_eq!(choose(&records), UndoChoice::Step { index: 2, step: UndoStep::SetIndex });
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
