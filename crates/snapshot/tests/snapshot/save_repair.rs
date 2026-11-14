//! WO-orc-snapshot-it-7b / S-41: real IndexNotSet -> unchanged save repairs actual index without more history.
use crate::support::{blob, run, write, Fixture};
use oplog::{oplog_path, read_op_log, OpPhase, OpResult};
use snapshot::{save, SnapshotError, DEFAULT_SIZE_CAP};
use std::fs;

#[test]
fn save_after_real_index_not_set_repairs_stale_real_index_with_no_commit_ref_or_log_movement() {
    // Joins save -> logged branch apply -> actual index.lock error -> retry's set_index -> exact tree/status.
    let f = Fixture::new(&[("removed.txt", b"old tracked file\n")]);
    write(&f.repo.path, "root.txt", b"new saved bytes\n");
    write(&f.repo.path, "added.txt", b"added saved bytes\n");
    fs::remove_file(f.repo.path.join("removed.txt")).unwrap();
    let old_index = fs::read(f.common.join("index")).unwrap();
    let old_tree = run(&f.git, &["write-tree"]);
    let lock_path = f.common.join("index.lock");
    fs::write(&lock_path, b"test-owned index lock\n").unwrap();
    let commit = match save(&f.git, &f.repo.path, &f.target, "Save before index repair", &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::IndexNotSet { commit, error } => {
            assert!(error.code.is_some_and(|code| code != 0));
            assert!(error.stderr.contains("index.lock"), "{error:?}");
            commit
        }
        other => panic!("fixture must reach the real post-branch index failure: {other:?}"),
    };
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(commit.clone()));
    let info = f.git.commit_info(&commit).unwrap();
    assert_eq!(info.parents, vec![f.head.clone()]);
    assert_eq!(blob(&f.git, &info.tree, "root.txt"), b"new saved bytes\n");
    assert_eq!(blob(&f.git, &info.tree, "added.txt"), b"added saved bytes\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), old_index);
    assert_ne!(old_tree, format!("{}\n", info.tree).as_bytes(), "precondition: the real index is genuinely stale");
    assert_eq!(fs::read(&lock_path).unwrap(), b"test-owned index lock\n");
    fs::remove_file(&lock_path).unwrap();
    assert!(!run(&f.git, &["status", "--porcelain=v1", "-z"]).is_empty(), "stale index cannot already be clean");
    let refs = run(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let log = fs::read(oplog_path(&f.common)).unwrap();
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 4);
    assert_eq!(records[3].kind, "save");
    assert_eq!(records[3].phase, OpPhase::Done);
    assert_eq!(records[3].result, Some(OpResult::Moved));
    assert_eq!(records[3].moves[0].new, commit);
    let commits = commit_ids(&f);
    let expected_entries = run(&f.git, &["ls-tree", "-r", "--format=%(objectmode) %(objectname) 0\t%(path)", "HEAD"]);
    // Explicit caller index remains untouched: both changed and unchanged save must target the folder's own index.
    let alternate = f.common.join("repair-caller-index");
    fs::write(&alternate, &old_index).unwrap();
    let mut caller = f.git.clone();
    caller.env.push(("GIT_INDEX_FILE".into(), alternate.clone().into_os_string()));
    let repaired = save(&caller, &f.repo.path, &f.target, "Retry without new history", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(!repaired.written);
    assert_eq!(repaired.commit, commit);
    assert_eq!(repaired.tree, info.tree);
    assert!(repaired.change_id.is_none());
    assert!(repaired.refused.is_empty());
    assert_eq!(run(&f.git, &["write-tree"]), format!("{}\n", info.tree).as_bytes());
    assert_eq!(run(&f.git, &["ls-files", "-s"]), expected_entries);
    assert!(run(&f.git, &["status", "--porcelain=v1", "-z"]).is_empty());
    assert_eq!(fs::read(&alternate).unwrap(), old_index);
    assert_eq!(run(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), refs);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log);
    assert_eq!(commit_ids(&f), commits);
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"new saved bytes\n");
    assert_eq!(fs::read(f.repo.path.join("added.txt")).unwrap(), b"added saved bytes\n");
    assert!(!f.repo.path.join("removed.txt").exists());
    assert!(!lock_path.exists());
}

fn commit_ids(f: &Fixture) -> Vec<String> {
    let bytes = run(&f.git, &["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]);
    let mut ids: Vec<_> = std::str::from_utf8(&bytes).unwrap().lines()
        .filter_map(|line| line.strip_prefix("commit ").map(str::to_owned)).collect();
    ids.sort(); ids
}
