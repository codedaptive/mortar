//! Joins undo/restore planners -> actual ref reads -> apply_protected/apply -> durable operation history.

use crate::support::{caller, refs, set_ref, test_op, Fixture};
use gitio::Oid;
use oplog::{apply, oplog_path, read_op_log, restore, undo, ApplyOutcome, OpPhase, OpRecord, OpResult, OplogError, RefMove};
use std::fs;

const A: &str = "refs/mortar/a";
const B: &str = "refs/mortar/b";
const C: &str = "refs/mortar/created";

#[test]
fn undo_reverses_create_update_and_delete_as_one_op_and_repeat_does_not_redo_it() {
    let f = Fixture::new();
    let zero = Oid::parse(&"0".repeat(40)).unwrap();
    let target = test_op("target", "original-agent", vec![
        movement(A, Some(&f.objects[0]), &f.objects[1]),
        movement(C, None, &f.objects[1]),
        movement(B, Some(&f.objects[0]), &zero),
    ]);
    done(apply(&f.git, &f.common, &target).unwrap());
    assert_values(&f, &[(A, Some(&f.objects[1])), (B, None), (C, Some(&f.objects[1]))]);
    let before = read_op_log(&f.common).unwrap().records;
    let undone = done(undo(&f.git, &f.common, &[A.into()], &caller("undo-request"), &[]).unwrap());
    assert_eq!(undone.moves, vec![
        movement(A, Some(&f.objects[1]), &f.objects[0]),
        movement(C, Some(&f.objects[1]), &zero),
        movement(B, None, &f.objects[0]),
    ]);
    assert_values(&f, &[(A, Some(&f.objects[0])), (B, Some(&f.objects[0])), (C, None)]);
    assert_one_transaction(&f, &before, &undone, "undo", "undo-request");

    // An undo is itself the latest done op, so a distinct request undoes it (redoes the original).
    let before_redo = read_op_log(&f.common).unwrap().records;
    let redone = done(undo(&f.git, &f.common, &[A.into()], &caller("redo-request"), &[]).unwrap());
    assert_values(&f, &[(A, Some(&f.objects[1])), (B, None), (C, Some(&f.objects[1]))]);
    assert_one_transaction(&f, &before_redo, &redone, "undo", "redo-request");
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(undo(&f.git, &f.common, &[A.into()], &caller("undo-request"), &[]).unwrap(), ApplyOutcome::Repeated(undone));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_values(&f, &[(A, Some(&f.objects[1])), (B, None), (C, Some(&f.objects[1]))]);
}

#[test]
fn undo_refuses_after_a_logged_foreign_move_naming_target_mover_and_actor() {
    let f = Fixture::new();
    let target = test_op("target", "original-agent", vec![
        movement(A, Some(&f.objects[0]), &f.objects[1]),
        movement(B, Some(&f.objects[0]), &f.objects[1]),
    ]);
    done(apply(&f.git, &f.common, &target).unwrap());
    let foreign = test_op("foreign-op", "foreign-agent", vec![movement(B, Some(&f.objects[1]), &f.objects[2])]);
    done(apply(&f.git, &f.common, &foreign).unwrap());
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let before = refs(&f);
    // Only A selects the target; the later op moved B, which that target also moved.
    let error = undo(&f.git, &f.common, &[A.into()], &caller("refused-undo"), &[]).unwrap_err();
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("target") && diagnostic.contains("foreign-op") && diagnostic.contains("foreign-agent"), "{diagnostic}");
    match error {
        OplogError::MovedSince { op_id, refname, left, actual, moved_by } => {
            assert_eq!(op_id, "target");
            assert_eq!(refname, B);
            assert_eq!(left, Some(f.objects[1].clone()));
            assert_eq!(actual, Some(f.objects[2].clone()));
            assert_eq!(moved_by, Some(("foreign-op".into(), "foreign-agent".into())));
        }
        other => panic!("expected MovedSince naming the logged foreign mover: {other:?}"),
    }
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(refs(&f), before);
}

#[test]
fn undo_refuses_an_unlogged_foreign_move_without_inventing_a_mover() {
    let f = Fixture::new();
    let target = test_op("target", "original-agent", vec![
        movement(A, Some(&f.objects[0]), &f.objects[1]),
        movement(B, Some(&f.objects[0]), &f.objects[1]),
    ]);
    done(apply(&f.git, &f.common, &target).unwrap());
    set_ref(&f.git, B, &f.objects[2]);
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let before = refs(&f);
    match undo(&f.git, &f.common, &[A.into()], &caller("refused-undo"), &[]).unwrap_err() {
        OplogError::MovedSince { op_id, refname, left, actual, moved_by } => {
            assert_eq!(op_id, "target");
            assert_eq!(refname, B);
            assert_eq!(left, Some(f.objects[1].clone()));
            assert_eq!(actual, Some(f.objects[2].clone()));
            assert_eq!(moved_by, None);
        }
        other => panic!("expected MovedSince for the external ref write: {other:?}"),
    }
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(refs(&f), before);
}

#[test]
fn restore_updates_creates_and_deletes_to_the_checkpoint_as_one_repeat_safe_op() {
    let f = Fixture::new();
    let zero = Oid::parse(&"0".repeat(40)).unwrap();
    let checkpoint = test_op("checkpoint", "original-agent", vec![
        movement(A, Some(&f.objects[0]), &f.objects[1]),
        movement(B, Some(&f.objects[0]), &zero),
        movement(C, None, &f.objects[1]),
    ]);
    done(apply(&f.git, &f.common, &checkpoint).unwrap());
    let later = test_op("later", "later-agent", vec![
        movement(A, Some(&f.objects[1]), &f.objects[2]),
        movement(B, None, &f.objects[2]),
        movement(C, Some(&f.objects[1]), &zero),
    ]);
    done(apply(&f.git, &f.common, &later).unwrap());
    let before = read_op_log(&f.common).unwrap().records;
    let selected = [A.into(), B.into(), C.into()];
    let restored = done(restore(&f.git, &f.common, "checkpoint", &selected, &caller("restore-request"), &[]).unwrap());
    assert_eq!(restored.moves, vec![
        movement(A, Some(&f.objects[2]), &f.objects[1]),
        movement(B, Some(&f.objects[2]), &zero),
        movement(C, None, &f.objects[1]),
    ]);
    assert_values(&f, &[(A, Some(&f.objects[1])), (B, None), (C, Some(&f.objects[1]))]);
    assert_one_transaction(&f, &before, &restored, "restore", "restore-request");
    set_ref(&f.git, A, &f.objects[2]);
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(restore(&f.git, &f.common, "checkpoint", &selected, &caller("restore-request"), &[]).unwrap(), ApplyOutcome::Repeated(restored));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_values(&f, &[(A, Some(&f.objects[2])), (B, None), (C, Some(&f.objects[1]))]);
}

fn movement(name: &str, old: Option<&Oid>, new: &Oid) -> RefMove {
    RefMove { refname: name.into(), old: old.cloned(), new: new.clone() }
}

fn done(outcome: ApplyOutcome) -> OpRecord {
    match outcome {
        ApplyOutcome::Done(record) => record,
        other => panic!("expected a fresh Done: {other:?}"),
    }
}

fn assert_values(f: &Fixture, expected: &[(&str, Option<&Oid>)]) {
    for (name, value) in expected {
        assert_eq!(f.git.read_ref(name).unwrap(), value.cloned(), "{name}");
    }
}

fn assert_one_transaction(f: &Fixture, before: &[OpRecord], closed: &OpRecord, kind: &str, key: &str) {
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), before.len() + 2, "exactly one intent and one close");
    assert_eq!(&log.records[..before.len()], before);
    let intent = &log.records[before.len()];
    assert_eq!(intent.phase, OpPhase::Intent);
    assert_eq!(intent.op_id, closed.op_id);
    assert_eq!(intent.moves, closed.moves);
    assert_eq!(log.records.last(), Some(closed));
    assert_eq!(closed.phase, OpPhase::Done);
    assert_eq!(closed.result, Some(OpResult::Moved));
    assert_eq!(closed.kind, kind);
    assert_eq!(closed.key, key);
    assert_eq!(closed.actor, "integration-requester");
    assert_eq!(closed.reason, "explicit integration request");
    assert!(!closed.op_id.is_empty());
    assert!(!before.iter().any(|r| r.op_id == closed.op_id));
}
