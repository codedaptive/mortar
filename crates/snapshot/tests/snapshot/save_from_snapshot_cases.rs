//! Saving a previously captured snapshot under the caller's lock.
use crate::support::{hook_script, run, tree_names, write, Fixture};
use snapshot::{save_from_snapshot, save_from_snapshot_with_hooks, snapshot, SaveHooksError, SnapshotError, DEFAULT_SIZE_CAP};
use std::fs;

#[test]
fn changed_snapshot_is_saved_without_another_snapshot() {
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"changed before snapshot\n");
    let built_on = f.git.head(&f.repo.path).unwrap().1.unwrap();
    let snap = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    let snap_tree = snap.tree.clone();
    let snap_commit = snap.commit.clone();
    write(&f.repo.path, "later.txt", b"added after snapshot\n");

    let saved = save_from_snapshot(&f.git, &f.repo.path, &f.target, "Save captured work", &built_on, snap).unwrap();
    assert!(saved.outcome.written);
    assert_eq!(saved.outcome.tree, snap_tree);
    assert!(!saved.snapshot_written);
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(saved.outcome.commit.clone()));
    let info = f.git.commit_info(&saved.outcome.commit).unwrap();
    assert_eq!(info.parents, vec![built_on]);
    assert_eq!(info.tree, snap_tree);
    assert_eq!(tree_names(&f.git, &info.tree), ["root.txt"]);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(snap_commit));
    assert_eq!(run(&f.git, &["status", "--porcelain=v1", "-z"]), b"?? later.txt\0");
    assert_eq!(fs::read(f.repo.path.join("later.txt")).unwrap(), b"added after snapshot\n");
}

#[test]
fn unchanged_snapshot_leaves_the_branch_at_built_on() {
    let f = Fixture::new(&[]);
    let built_on = f.git.head(&f.repo.path).unwrap().1.unwrap();
    let snap = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    let snap_commit = snap.commit.clone();
    write(&f.repo.path, "later.txt", b"added after snapshot\n");

    let saved = save_from_snapshot(&f.git, &f.repo.path, &f.target, "Save unchanged work", &built_on, snap).unwrap();
    assert!(!saved.outcome.written);
    assert_eq!(saved.outcome.commit, built_on);
    assert!(saved.outcome.change_id.is_none());
    assert!(!saved.snapshot_written);
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(built_on.clone()));
    assert_eq!(f.git.read_ref("refs/heads/main").unwrap(), Some(built_on));
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(snap_commit));
    assert_eq!(run(&f.git, &["status", "--porcelain=v1", "-z"]), b"?? later.txt\0");
}

#[test]
fn moved_branch_refuses_changed_snapshot_and_preserves_foreign_commit_and_files() {
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"our captured work\n");
    let built_on = f.git.head(&f.repo.path).unwrap().1.unwrap();
    let snap = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    write(&f.repo.path, "foreign.txt", b"foreign work\n");
    run(&f.git, &["add", "foreign.txt"]);
    run(&f.git, &["commit", "-m", "Foreign commit"]);
    let foreign = f.git.rev_parse("HEAD").unwrap().unwrap();
    assert_ne!(foreign, built_on);

    let error = save_from_snapshot(&f.git, &f.repo.path, &f.target, "Save captured work", &built_on, snap).unwrap_err();
    assert!(matches!(error, SnapshotError::BranchMoved(_)), "{error:?}");
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(foreign));
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"our captured work\n");
    assert_eq!(fs::read(f.repo.path.join("foreign.txt")).unwrap(), b"foreign work\n");
}

#[test]
fn moved_branch_refuses_unchanged_snapshot_without_resetting_index() {
    let f = Fixture::new(&[]);
    let built_on = f.git.head(&f.repo.path).unwrap().1.unwrap();
    let snap = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    write(&f.repo.path, "foreign.txt", b"staged foreign tree\n");
    run(&f.git, &["add", "foreign.txt"]);
    fs::remove_file(f.repo.path.join("foreign.txt")).unwrap();
    let index_before = run(&f.git, &["write-tree"]);
    assert_ne!(index_before, run(&f.git, &["rev-parse", "HEAD^{tree}"]));
    let tree = std::str::from_utf8(&index_before).unwrap().trim();
    let foreign = f.git.commit_tree(&gitio::Oid::parse(tree).unwrap(), std::slice::from_ref(&built_on), "Foreign commit\n", None, None).unwrap();
    run(&f.git, &["update-ref", "refs/heads/main", foreign.as_str(), built_on.as_str()]);

    let error = save_from_snapshot(&f.git, &f.repo.path, &f.target, "Save unchanged work", &built_on, snap).unwrap_err();
    assert!(matches!(error, SnapshotError::BranchMoved(_)), "{error:?}");
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(foreign));
    assert_eq!(run(&f.git, &["write-tree"]), index_before);
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"base\n");
    assert!(!f.repo.path.join("foreign.txt").exists());
}

#[test]
fn hooks_refusing_pre_commit_stops_save_from_snapshot_and_leaves_the_branch() {
    // Case 7.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"captured then refused\n");
    let built_on = f.git.head(&f.repo.path).unwrap().1.unwrap();
    let snap = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    hook_script(&f, "pre-commit", b"exit 1\n");
    match save_from_snapshot_with_hooks(&f.git, &f.repo.path, &f.target, "Refused captured work", &built_on, snap) {
        Err(SaveHooksError::HookRefused(refusal)) => assert_eq!((refusal.hook, refusal.code), ("pre-commit", Some(1))),
        other => panic!("expected HookRefused, got {other:?}"),
    }
    assert_eq!(f.git.read_ref("refs/heads/main").unwrap(), Some(built_on.clone()));
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(built_on));
}
