use crate::support::{kill_apply, operation, refs, Fixture};
use gitio::{Git, Oid};
use oplog::{apply, oplog_path, read_op_log, recover, ApplyOutcome, OpPhase, OpResult};
use std::fs;

#[test]
fn killed_between_intent_and_transaction_recovers_failed_then_retry_moves_once() {
    // Joins apply -> durable intent -> SIGKILL -> recover -> retry through real Git::update_refs.
    let f = Fixture::new();
    let op = operation(&f.objects);
    kill_apply(&f, "before");
    assert_open_intent(&f);
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[0]));
    let closed = recover(&f.repo.git, &f.common).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].phase, OpPhase::Failed);
    assert_eq!(closed[0].result, Some(OpResult::NotMoved));
    assert_eq!(read_op_log(&f.common).unwrap().records[1], closed[0]);
    assert!(matches!(apply(&f.repo.git, &f.common, &op).unwrap(), ApplyOutcome::Done(_)));
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[1]));
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(read_op_log(&f.common).unwrap().records.len(), 4);
    assert!(matches!(apply(&f.repo.git, &f.common, &op).unwrap(), ApplyOutcome::Repeated(_)));
    assert!(recover(&f.repo.git, &f.common).unwrap().is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn killed_after_transaction_before_done_recovers_done_and_repeat_appends_nothing() {
    // Joins real update-ref commit -> SIGKILL -> recover/read_op_log -> apply's repeat check.
    let f = Fixture::new();
    let op = operation(&f.objects);
    kill_apply(&f, "after");
    assert_open_intent(&f);
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[1]));
    let closed = recover(&f.repo.git, &f.common).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].phase, OpPhase::Done);
    assert_eq!(closed[0].result, Some(OpResult::Moved));
    assert_eq!(read_op_log(&f.common).unwrap().records[1], closed[0]);
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(apply(&f.repo.git, &f.common, &op).unwrap(), ApplyOutcome::Repeated(closed[0].clone()));
    assert!(recover(&f.repo.git, &f.common).unwrap().is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[1]));
}

fn assert_open_intent(f: &Fixture) {
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 1, "worker must die before any closing record");
    assert_eq!(records[0].phase, OpPhase::Intent);
    assert_eq!(records[0].op_id, "integration-op-1");
    assert_eq!(records[0].key, "integration-repeat-key");
    assert_eq!(records[0].moves, operation(&f.objects).moves);
    assert_eq!(records[0].result, None);
}

#[test]
#[ignore = "SIGKILL subprocess entry point; exercised explicitly by the parent crash tests"]
fn apply_worker() {
    let path = std::path::PathBuf::from(std::env::var_os("MORTAR_TEST_REPO").unwrap());
    let objects = ["MORTAR_TEST_OLD", "MORTAR_TEST_NEW", "MORTAR_TEST_FOREIGN"]
        .map(|key| Oid::parse(&std::env::var(key).unwrap()).unwrap());
    let mut git = Git::new(&path);
    git.program = path.join("git-crash-boundary.sh");
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    apply(&git, &path.join(".git"), &operation(&objects)).unwrap();
    panic!("crash shim returned without killing apply");
}
