use crate::support::{blob, run, tree_names, write, Fixture};
use gitio::Git;
use oplog::{oplog_path, read_op_log, OpPhase, OpResult, RefMove};
use snapshot::{is_saved, snapshot, DEFAULT_SIZE_CAP};
use std::fs;

#[test]
fn first_snapshot_saves_real_tree_parent_trailers_ref_and_attributed_op() {
    // Joins snapshot/is_saved -> gitio tree/commit/ref operations -> oplog apply/read_op_log.
    let f = Fixture::new(&[]);
    assert!(!is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert!(!oplog_path(&f.common).exists());
    write(&f.repo.path, "root.txt", b"first change\n");
    write(&f.repo.path, "new.txt", b"new file\n");
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(saved.written);
    assert!(saved.refused.is_empty());
    assert_eq!(tree_names(&f.git, &saved.tree), ["new.txt", "root.txt"]);
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"first change\n");
    assert_eq!(blob(&f.git, &saved.tree, "new.txt"), b"new file\n");
    let info = f.git.commit_info(&saved.commit).unwrap();
    assert_eq!(info.tree, saved.tree);
    assert_eq!(info.parents, vec![f.head.clone()]);
    assert_eq!(info.message, "snapshot\n\nMortar-Request: request-17\nMortar-Zone: snapshot-zone\n");
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(saved.commit.clone()));
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.records[0].phase, OpPhase::Intent);
    assert_eq!(log.records[0].result, None);
    assert_eq!(log.records[1].phase, OpPhase::Done);
    assert_eq!(log.records[1].result, Some(OpResult::Moved));
    for record in &log.records {
        assert_eq!(record.kind, "snapshot");
        assert_eq!(record.reason, "snapshot");
        assert_eq!(record.actor, "snapshot-caller");
        assert_eq!(record.moves, vec![RefMove { refname: f.refname.clone(), old: None, new: saved.commit.clone() }]);
        assert!(!record.op_id.is_empty());
    }
    assert_eq!(log.records[0].op_id, log.records[1].op_id);
    assert_eq!(log.records[0].key, log.records[1].key);
    let before = fs::read(oplog_path(&f.common)).unwrap();
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before);
}

#[test]
fn unchanged_writes_nothing_then_changed_snapshot_chains_head_and_previous() {
    // Joins snapshot's tree equality -> immutable object/ref/log observation -> subsequent oplog apply.
    let f = Fixture::new(&[]);
    let first = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    let log_before = fs::read(oplog_path(&f.common)).unwrap();
    let objects_before = object_ids(&f.git);
    let again = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(!again.written);
    assert_eq!(again.commit, first.commit);
    assert_eq!(again.tree, first.tree);
    assert!(again.refused.is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    assert_eq!(object_ids(&f.git), objects_before);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(first.commit.clone()));
    write(&f.repo.path, "root.txt", b"second change\n");
    assert!(!is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    let second = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(second.written);
    assert_ne!(second.commit, first.commit);
    assert_eq!(blob(&f.git, &second.tree, "root.txt"), b"second change\n");
    assert_eq!(f.git.commit_info(&second.commit).unwrap().parents, vec![f.head.clone(), first.commit.clone()]);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(second.commit.clone()));
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 4);
    assert_eq!(records[2].phase, OpPhase::Intent);
    assert_eq!(records[3].phase, OpPhase::Done);
    assert_eq!(records[3].result, Some(OpResult::Moved));
    assert_eq!(records[3].moves, vec![RefMove { refname: f.refname.clone(), old: Some(first.commit), new: second.commit }]);
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
}

fn object_ids(git: &Git) -> Vec<String> {
    let bytes = run(git, &["cat-file", "--batch-all-objects", "--batch-check=%(objectname)"]);
    let mut ids: Vec<_> = std::str::from_utf8(&bytes).unwrap().lines().map(str::to_owned).collect();
    ids.sort();
    ids
}
