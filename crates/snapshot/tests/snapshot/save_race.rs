//! WO-orc-snapshot-it-6: save's branch CAS joins a real competing oplog writer under the caller's existing lock.
use crate::support::{blob, run, write, Fixture};
use gitio::{Git, Oid};
use oplog::{apply, decode_lock_record, read_op_log, ApplyOutcome, Op, OpPhase, OpResult, RefMove};
use snapshot::{save, SnapshotError, DEFAULT_SIZE_CAP};
use std::{env, fs, path::PathBuf};
use testkit::write_git_shim;

#[test]
fn branch_moved_during_save_is_named_without_overwriting_foreign_head_index_work_or_caller_lock() {
    // Joins save's captured HEAD -> actual foreign apply -> branch expected-old refusal and durable failed record.
    let f = Fixture::new(&[]);
    let tree = Oid::parse(std::str::from_utf8(&run(&f.git, &["rev-parse", "HEAD^{tree}"])).unwrap().trim_end()).unwrap();
    let foreign = f.git.commit_tree(&tree, std::slice::from_ref(&f.head), "foreign branch commit\n", None, None).unwrap();
    write(&f.repo.path, "root.txt", b"our unsaved branch candidate\n");
    let index = fs::read(f.common.join("index")).unwrap();
    let lock = fs::read(&f.lock.path).unwrap();
    let shim = f.common.join("save-race.sh");
    let worker_log = f.common.join("save-race-worker.log");
    // The first commit-tree writes the snapshot; the second is save's branch candidate, after its HEAD read.
    write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = commit-tree ]; then\n  if mkdir \"$MORTAR_SAVE_FIRST\" 2>/dev/null; then\n    :\n  elif mkdir \"$MORTAR_SAVE_ONCE\" 2>/dev/null; then\n    \"$MORTAR_SAVE_EXE\" --exact save_race::foreign_save_worker --ignored --nocapture >\"$MORTAR_SAVE_LOG\" 2>&1\n  fi\nfi\nexec git \"$@\"\n").unwrap();
    let mut racing = f.git.clone();
    racing.program = shim;
    racing.env.extend([
        ("MORTAR_SAVE_FIRST".into(), f.common.join("save-first-commit").into_os_string()),
        ("MORTAR_SAVE_ONCE".into(), f.common.join("save-race-once").into_os_string()),
        ("MORTAR_SAVE_EXE".into(), env::current_exe().unwrap().into_os_string()),
        ("MORTAR_SAVE_LOG".into(), worker_log.clone().into_os_string()),
        ("MORTAR_SAVE_REPO".into(), f.repo.path.clone().into_os_string()),
        ("MORTAR_SAVE_COMMON".into(), f.common.clone().into_os_string()),
        ("MORTAR_SAVE_LOCK".into(), f.lock.path.clone().into_os_string()),
        ("MORTAR_SAVE_CALLER".into(), std::process::id().to_string().into()),
        ("MORTAR_SAVE_OLD".into(), f.head.to_string().into()),
        ("MORTAR_SAVE_NEW".into(), foreign.to_string().into()),
    ]);
    let failed = match save(&racing, &f.repo.path, &f.target, "Save with a racing branch", &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::BranchMoved(record) => record,
        other => panic!("expected BranchMoved, got {other:?}"),
    };
    let output = fs::read_to_string(worker_log).unwrap();
    assert!(output.contains("1 passed"), "{output}");
    assert_eq!(failed.phase, OpPhase::Failed);
    assert_eq!(failed.kind, "save");
    assert_eq!(failed.result, Some(OpResult::Stale { refname: "refs/heads/main".into(), actual: Some(foreign.clone()) }));
    assert_eq!(failed.moves.len(), 1);
    assert_eq!(failed.moves[0].refname, "refs/heads/main");
    assert_eq!(failed.moves[0].old, Some(f.head.clone()));
    let candidate = f.git.commit_info(&failed.moves[0].new).unwrap();
    assert_eq!(candidate.parents, vec![f.head.clone()]);
    assert_eq!(blob(&f.git, &candidate.tree, "root.txt"), b"our unsaved branch candidate\n");
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(foreign.clone()));
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index);
    assert_eq!(fs::read(&f.lock.path).unwrap(), lock);
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"our unsaved branch candidate\n");
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 6);
    assert_eq!(log.records[3].actor, "foreign-save-writer");
    assert_eq!(log.records[3].phase, OpPhase::Done);
    assert_eq!(log.records[3].moves[0].new, foreign);
    assert_eq!(log.records[5], failed);
}

#[test]
#[ignore = "actual competing branch writer, invoked synchronously by the save-race parent at its commit boundary"]
fn foreign_save_worker() {
    let common = PathBuf::from(env::var_os("MORTAR_SAVE_COMMON").unwrap());
    let lock = fs::read_to_string(PathBuf::from(env::var_os("MORTAR_SAVE_LOCK").unwrap())).unwrap();
    let holder = decode_lock_record(&lock).unwrap();
    assert_eq!(holder.pid.to_string(), env::var("MORTAR_SAVE_CALLER").unwrap());
    assert!(oplog::PidProbe::pid_alive(&oplog::KillPidProbe, holder.pid));
    let mut git = Git::new(PathBuf::from(env::var_os("MORTAR_SAVE_REPO").unwrap()));
    git.env.extend([("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into())]);
    let op = Op {
        op_id: "foreign-save-op".into(), key: "foreign-save-key".into(), kind: "save".into(), actor: "foreign-save-writer".into(),
        reason: "deterministic foreign branch movement".into(), at: 101,
        moves: vec![RefMove { refname: "refs/heads/main".into(),
            old: Some(Oid::parse(&env::var("MORTAR_SAVE_OLD").unwrap()).unwrap()),
            new: Oid::parse(&env::var("MORTAR_SAVE_NEW").unwrap()).unwrap() }],
    };
    // Deliberately models an uncooperative writer; the parent retains its real caller lock throughout.
    assert!(matches!(apply(&git, &common, &op).unwrap(), ApplyOutcome::Done(_)));
}
