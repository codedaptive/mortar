use crate::support::{blob, write, Fixture};
use gitio::{Git, Oid};
use oplog::{apply, decode_lock_record, oplog_path, read_op_log, ApplyOutcome, Op, OpPhase, OpResult, RefMove};
use snapshot::{snapshot, SnapshotError, DEFAULT_SIZE_CAP};
use std::{env, fs, path::PathBuf};
use testkit::write_git_shim;

#[test]
fn foreign_logged_move_after_previous_read_is_refused_without_overwriting_ref_or_files() {
    // Joins snapshot -> previous-ref read -> real competing oplog apply -> snapshot's expected-old CAS.
    // The caller holds the zone lock throughout. The injected foreign operation deliberately models
    // an uncooperative writer; snapshot must not take the caller's lock or overwrite the foreign ref.
    let f = Fixture::new(&[]);
    let first = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    let foreign = f.git.commit_tree(&first.tree, &[f.head.clone()], "foreign save\n", None, None).unwrap();
    write(&f.repo.path, "root.txt", b"snapshot change that must remain on disk\n");
    let index_before = fs::read(f.common.join("index")).unwrap();
    let lock_before = fs::read(&f.lock.path).unwrap();
    let shim = f.common.join("snapshot-race.sh");
    let worker_log = f.common.join("snapshot-race.log");
    // commit-tree is reached only after snapshot has read the old snap ref and its tree.
    // The real helper must finish its apply before the original git invocation continues.
    write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = commit-tree ]; then\n  \"$MORTAR_RACE_EXE\" --exact race::foreign_move_worker --ignored --nocapture >\"$MORTAR_RACE_LOG\" 2>&1\nfi\nexec git \"$@\"\n").unwrap();
    let mut racing = f.git.clone();
    racing.program = shim;
    racing.env.extend([
        ("MORTAR_RACE_EXE".into(), env::current_exe().unwrap().into_os_string()),
        ("MORTAR_RACE_LOG".into(), worker_log.clone().into_os_string()),
        ("MORTAR_RACE_REPO".into(), f.repo.path.clone().into_os_string()),
        ("MORTAR_RACE_COMMON".into(), f.common.clone().into_os_string()),
        ("MORTAR_RACE_REF".into(), f.refname.clone().into()),
        ("MORTAR_RACE_OLD".into(), first.commit.to_string().into()),
        ("MORTAR_RACE_NEW".into(), foreign.to_string().into()),
        ("MORTAR_RACE_LOCK".into(), f.lock.path.clone().into_os_string()),
        ("MORTAR_RACE_CALLER_PID".into(), std::process::id().to_string().into()),
    ]);
    let failed = match snapshot(&racing, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::RefMoved(record) => record,
        other => panic!("expected RefMoved from oplog CAS, got {other:?}"),
    };
    let worker_output = fs::read_to_string(worker_log).unwrap();
    assert!(worker_output.contains("1 passed"), "{worker_output}");
    assert_eq!(fs::read(f.common.join("foreign-op-applied")).unwrap(), b"done\n");
    assert_eq!(failed.phase, OpPhase::Failed);
    assert_eq!(failed.result, Some(OpResult::Stale { refname: f.refname.clone(), actual: Some(foreign.clone()) }));
    assert_eq!(failed.moves.len(), 1);
    assert_eq!(failed.moves[0].refname, f.refname);
    assert_eq!(failed.moves[0].old, Some(first.commit.clone()));
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(foreign.clone()));
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"snapshot change that must remain on disk\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    assert_eq!(fs::read(&f.lock.path).unwrap(), lock_before);
    // Even the rejected candidate is a real intact commit; only the guarded ref movement refused.
    let candidate = f.git.commit_info(&failed.moves[0].new).unwrap();
    assert_eq!(candidate.parents, vec![f.head.clone(), first.commit.clone()]);
    assert_eq!(blob(&f.git, &candidate.tree, "root.txt"), b"snapshot change that must remain on disk\n");
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 6);
    assert_eq!(log.records[2].phase, OpPhase::Intent);
    assert_eq!(log.records[3].phase, OpPhase::Done);
    assert_eq!(log.records[3].result, Some(OpResult::Moved));
    assert_eq!(log.records[3].actor, "foreign-writer");
    assert_eq!(log.records[3].moves, vec![RefMove { refname: f.refname.clone(), old: Some(first.commit), new: foreign }]);
    assert_eq!(log.records[4].phase, OpPhase::Intent);
    assert_eq!(log.records[4].op_id, failed.op_id);
    assert_eq!(log.records[5], failed);
    let before = fs::read(oplog_path(&f.common)).unwrap();
    assert!(oplog::recover(&f.git, &f.common).unwrap().is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before);
}

#[test]
#[ignore = "real competing operation, run synchronously by the parent git shim at the read/apply boundary"]
fn foreign_move_worker() {
    let repo = PathBuf::from(env::var_os("MORTAR_RACE_REPO").unwrap());
    let common = PathBuf::from(env::var_os("MORTAR_RACE_COMMON").unwrap());
    let lock = fs::read_to_string(PathBuf::from(env::var_os("MORTAR_RACE_LOCK").unwrap())).unwrap();
    let holder = decode_lock_record(&lock).unwrap();
    assert_eq!(holder.pid.to_string(), env::var("MORTAR_RACE_CALLER_PID").unwrap());
    assert!(oplog::PidProbe::pid_alive(&oplog::KillPidProbe, holder.pid));
    let mut git = Git::new(&repo);
    git.env = vec![("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into())];
    let op = Op {
        op_id: "foreign-snapshot-op".into(), key: "foreign-snapshot-key".into(),
        kind: "snapshot".into(), actor: "foreign-writer".into(), reason: "race at snapshot read/apply".into(), at: 101,
        moves: vec![RefMove {
            refname: env::var("MORTAR_RACE_REF").unwrap(),
            old: Some(Oid::parse(&env::var("MORTAR_RACE_OLD").unwrap()).unwrap()),
            new: Oid::parse(&env::var("MORTAR_RACE_NEW").unwrap()).unwrap(),
        }],
    };
    assert!(matches!(apply(&git, &common, &op).unwrap(), ApplyOutcome::Done(_)));
    assert_eq!(git.read_ref(&op.moves[0].refname).unwrap(), Some(op.moves[0].new.clone()));
    fs::write(common.join("foreign-op-applied"), b"done\n").unwrap();
}
