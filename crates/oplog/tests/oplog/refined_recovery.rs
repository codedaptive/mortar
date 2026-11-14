//! S-13: joins real intent persistence, real refs, recovery decisions and durable closing records.

use crate::support::{kill_apply, operation, refs, set_ref, Fixture};
use gitio::Oid;
use oplog::{
    append_op_record, intent_record, oplog_path, read_op_log, recover,
    Op, OpPhase, OpResult, OplogError, RefMove,
};
use std::fs;

#[test]
fn recovery_with_no_ref_at_new_closes_failed_even_when_a_ref_is_not_at_old() {
    // Restores the deferred real-SIGKILL case on the shared, reaped-worker testkit gate.
    let f = Fixture::new();
    set_ref(&f.git, "refs/mortar/b", &f.objects[2]);
    kill_apply(&f, "before");
    let op = operation(&f.objects);
    let before = format!("refs/mortar/a {}\nrefs/mortar/b {}\n", f.objects[0], f.objects[2]);
    assert_eq!(refs(&f), before);
    assert_eq!(read_op_log(&f.common).unwrap().records, vec![intent_record(&op)]);
    assert_closes(&f, &op, OpPhase::Failed, OpResult::NotMoved { refs: vec![
        ("refs/mortar/a".into(), Some(f.objects[0].clone())),
        ("refs/mortar/b".into(), Some(f.objects[2].clone())),
    ] });
    assert_eq!(refs(&f), before);
}

#[test]
fn unchanged_moves_at_new_cannot_turn_a_never_run_transaction_into_a_mix() {
    // Changing refs are old/foreign, unchanged refs are at new, including an absent-ref delete.
    let f = Fixture::new();
    let mut op = operation(&f.objects);
    set_ref(&f.git, "refs/mortar/b", &f.objects[2]);
    add_noop_moves(&f, &mut op);
    set_ref(&f.git, "refs/mortar/unchanged", &f.objects[0]);
    append_op_record(&f.common, &intent_record(&op)).unwrap();
    assert_closes(&f, &op, OpPhase::Failed, OpResult::NotMoved { refs: vec![
        ("refs/mortar/a".into(), Some(f.objects[0].clone())),
        ("refs/mortar/b".into(), Some(f.objects[2].clone())),
        ("refs/mortar/unchanged".into(), Some(f.objects[0].clone())),
        ("refs/mortar/absent-delete".into(), None),
    ] });
}

#[test]
fn unchanged_moves_holding_foreign_values_do_not_prevent_failed_recovery() {
    let f = Fixture::new();
    let mut op = operation(&f.objects);
    add_noop_moves(&f, &mut op);
    for name in ["refs/mortar/unchanged", "refs/mortar/absent-delete"] {
        set_ref(&f.git, name, &f.objects[2]);
    }
    append_op_record(&f.common, &intent_record(&op)).unwrap();
    assert_closes(&f, &op, OpPhase::Failed, OpResult::NotMoved { refs: vec![
        ("refs/mortar/a".into(), Some(f.objects[0].clone())),
        ("refs/mortar/b".into(), Some(f.objects[0].clone())),
        ("refs/mortar/unchanged".into(), Some(f.objects[2].clone())),
        ("refs/mortar/absent-delete".into(), Some(f.objects[2].clone())),
    ] });
}

#[test]
fn all_changing_moves_at_new_close_done_despite_foreign_unchanged_moves() {
    let f = Fixture::new();
    let mut op = operation(&f.objects);
    add_noop_moves(&f, &mut op);
    for name in ["refs/mortar/a", "refs/mortar/b"] {
        set_ref(&f.git, name, &f.objects[1]);
    }
    for name in ["refs/mortar/unchanged", "refs/mortar/absent-delete"] {
        set_ref(&f.git, name, &f.objects[2]);
    }
    append_op_record(&f.common, &intent_record(&op)).unwrap();
    assert_closes(&f, &op, OpPhase::Done, OpResult::Moved);
}

#[test]
fn a_true_mix_of_changing_moves_is_refused_with_every_actual_value() {
    let f = Fixture::new();
    let mut op = operation(&f.objects);
    add_noop_moves(&f, &mut op);
    set_ref(&f.git, "refs/mortar/a", &f.objects[1]);
    for name in ["refs/mortar/unchanged", "refs/mortar/absent-delete"] {
        set_ref(&f.git, name, &f.objects[2]);
    }
    append_op_record(&f.common, &intent_record(&op)).unwrap();
    let log = fs::read(oplog_path(&f.common)).unwrap();
    let before = refs(&f);
    match recover(&f.git, &f.common).unwrap_err() {
        OplogError::Mixed { op_id, key, refs } => {
            assert_eq!(op_id, "integration-op-1");
            assert_eq!(key, "integration-repeat-key");
            assert_eq!(refs, vec![
                (op.moves[0].clone(), Some(f.objects[1].clone())),
                (op.moves[1].clone(), Some(f.objects[0].clone())),
                (op.moves[2].clone(), Some(f.objects[2].clone())),
                (op.moves[3].clone(), Some(f.objects[2].clone())),
            ]);
        }
        other => panic!("expected true Mixed: {other:?}"),
    }
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log);
    assert_eq!(refs(&f), before);
}

#[test]
fn all_unchanged_moves_close_done_even_when_their_actual_values_are_foreign() {
    let f = Fixture::new();
    let mut op = operation(&f.objects);
    op.moves.clear();
    add_noop_moves(&f, &mut op);
    for name in ["refs/mortar/unchanged", "refs/mortar/absent-delete"] {
        set_ref(&f.git, name, &f.objects[2]);
    }
    append_op_record(&f.common, &intent_record(&op)).unwrap();
    assert_closes(&f, &op, OpPhase::Done, OpResult::Moved);
}

fn add_noop_moves(f: &Fixture, op: &mut Op) {
    op.moves.push(RefMove {
        refname: "refs/mortar/unchanged".into(),
        old: Some(f.objects[0].clone()), new: f.objects[0].clone(),
    });
    op.moves.push(RefMove {
        refname: "refs/mortar/absent-delete".into(),
        old: None, new: Oid::parse(&"0".repeat(40)).unwrap(),
    });
}

fn assert_closes(f: &Fixture, op: &Op, phase: OpPhase, result: OpResult) {
    let before = refs(f);
    let closed = recover(&f.git, &f.common).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].phase, phase);
    assert_eq!(closed[0].result, Some(result));
    assert_eq!(closed[0].op_id, "integration-op-1");
    assert_eq!(closed[0].key, "integration-repeat-key");
    assert_eq!(closed[0].moves, op.moves);
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records, vec![intent_record(op), closed[0].clone()]);
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert!(recover(&f.git, &f.common).unwrap().is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(refs(f), before, "recovery must not move refs");
}
