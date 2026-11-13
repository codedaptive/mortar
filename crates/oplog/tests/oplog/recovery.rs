use crate::support::{kill_apply, operation, refs, set_ref, Fixture};
use oplog::{oplog_path, read_op_log, recover, OplogError};
use std::fs;

#[test]
fn true_mixed_recovery_refuses_and_appends_nothing() {
    // Joins killed apply -> foreign ref write -> recover's mixed refusal, preserving the intent.
    let f = Fixture::new();
    kill_apply(&f, "after");
    set_ref(&f.repo.git, "refs/mortar/b", &f.objects[2]);
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    match recover(&f.repo.git, &f.common).unwrap_err() {
        OplogError::Mixed { op_id, key, refs } => {
            assert_eq!(op_id, "integration-op-1");
            assert_eq!(key, "integration-repeat-key");
            assert_eq!(refs, vec![
                (operation(&f.objects).moves[0].clone(), Some(f.objects[1].clone())),
                (operation(&f.objects).moves[1].clone(), Some(f.objects[2].clone())),
            ]);
        }
        other => panic!("expected Mixed: {other:?}"),
    }
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(refs(&f), format!("refs/mortar/a {}\nrefs/mortar/b {}\n", f.objects[1], f.objects[2]));
}

#[test]
fn recovery_of_missing_log_is_empty_and_creates_nothing() {
    // Joins recover -> read_op_log on a real git common directory without a log.
    let f = Fixture::new();
    assert!(recover(&f.repo.git, &f.common).unwrap().is_empty());
    assert!(!oplog_path(&f.common).exists());
    assert!(read_op_log(&f.common).unwrap().records.is_empty());
}
