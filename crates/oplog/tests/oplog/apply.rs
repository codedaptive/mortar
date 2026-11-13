use crate::support::{operation, refs, set_ref, Fixture};
use oplog::{apply, oplog_path, read_op_log, ApplyOutcome, OpPhase, OpResult};
use std::fs;

#[test]
fn apply_persists_intent_then_done_and_same_key_moves_only_once() {
    // Joins apply -> Git::update_refs -> append_op_record/read_op_log, with real refs.
    let f = Fixture::new();
    let op = operation(&f.objects);
    assert!(!oplog_path(&f.common).exists());
    let done = match apply(&f.repo.git, &f.common, &op).unwrap() {
        ApplyOutcome::Done(record) => record,
        other => panic!("expected Done: {other:?}"),
    };
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[1]));
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.records[0].phase, OpPhase::Intent);
    assert_eq!(log.records[0].result, None);
    assert_eq!(log.records[1], done);
    assert_eq!(done.phase, OpPhase::Done);
    assert_eq!(done.result, Some(OpResult::Moved));
    for record in &log.records {
        assert_eq!(record.op_id, "integration-op-1");
        assert_eq!(record.key, "integration-repeat-key");
        assert_eq!(record.actor, "integration-actor");
        assert_eq!(record.kind, "join");
        assert_eq!(record.reason, "move two refs atomically");
        assert_eq!(record.moves, op.moves);
    }
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    // A foreign move makes rerunning the transaction observably wrong, even if it would
    // merely log a stale failure rather than move a ref twice.
    for name in ["refs/mortar/a", "refs/mortar/b"] {
        set_ref(&f.repo.git, name, &f.objects[2]);
    }
    let mut repeated = op.clone();
    repeated.op_id = "different-attempt-same-key".into();
    repeated.moves[0].new = f.objects[0].clone();
    assert_eq!(apply(&f.repo.git, &f.common, &repeated).unwrap(), ApplyOutcome::Repeated(done));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[2]));
}

#[test]
fn stale_apply_moves_nothing_and_records_ref_and_actual_value() {
    // Joins apply -> Git::update_refs -> read_ref_value -> failed log append/readback.
    let f = Fixture::new();
    let op = operation(&f.objects);
    set_ref(&f.repo.git, "refs/mortar/b", &f.objects[2]);
    let failed = match apply(&f.repo.git, &f.common, &op).unwrap() {
        ApplyOutcome::Failed(record) => record,
        other => panic!("expected Failed: {other:?}"),
    };
    assert_eq!(failed.phase, OpPhase::Failed);
    assert_eq!(failed.result, Some(OpResult::Stale {
        refname: "refs/mortar/b".into(),
        actual: Some(f.objects[2].clone()),
    }));
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].phase, OpPhase::Intent);
    assert_eq!(records[1], failed);
    assert_eq!(refs(&f), format!("refs/mortar/a {}\nrefs/mortar/b {}\n", f.objects[0], f.objects[2]));
}
