use crate::support::{blob, tree_names, write, Fixture};
use oplog::oplog_path;
use snapshot::{is_saved, snapshot, RefusedFile, DEFAULT_SIZE_CAP};
use std::{fs, path::PathBuf};

#[test]
fn new_six_mib_file_is_reported_but_large_changed_head_file_is_kept() {
    // Joins snapshot -> folder_tree size/HEAD membership -> gitio capped tree -> oplog apply.
    let old = vec![b'h'; 6 * 1024 * 1024];
    let changed = vec![b'c'; 6 * 1024 * 1024];
    let new = vec![b'n'; 6 * 1024 * 1024];
    let f = Fixture::new(&[("large-tracked", &old)]);
    write(&f.repo.path, "large-tracked", &changed);
    write(&f.repo.path, "large-new", &new);
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(saved.written);
    assert_eq!(saved.refused, vec![RefusedFile { path: PathBuf::from("large-new"), size: 6 * 1024 * 1024, cap: 5 * 1024 * 1024 }]);
    assert_eq!(tree_names(&f.git, &saved.tree), ["large-tracked", "root.txt"]);
    assert_eq!(blob(&f.git, &saved.tree, "large-tracked"), changed);
    assert_eq!(fs::read(f.repo.path.join("large-tracked")).unwrap(), changed);
    assert_eq!(fs::read(f.repo.path.join("large-new")).unwrap(), new);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(saved.commit));
    assert_eq!(oplog::read_op_log(&f.common).unwrap().records.len(), 2);
}

#[test]
fn refused_only_change_is_saved_and_unchanged_outcome_still_reports_the_refusal() {
    // Joins is_saved and snapshot -> the same capped tree -> no extra oplog/ref movement (S-31).
    let f = Fixture::new(&[]);
    let first = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    let log_before = fs::read(oplog_path(&f.common)).unwrap();
    let new = vec![b'n'; 6 * 1024 * 1024];
    write(&f.repo.path, "refused-only", &new);
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    let again = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(!again.written);
    assert_eq!(again.commit, first.commit);
    assert_eq!(again.tree, first.tree);
    assert_eq!(again.refused, vec![RefusedFile { path: PathBuf::from("refused-only"), size: 6 * 1024 * 1024, cap: 5 * 1024 * 1024 }]);
    assert_eq!(tree_names(&f.git, &again.tree), ["root.txt"]);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(first.commit));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    assert_eq!(fs::read(f.repo.path.join("refused-only")).unwrap(), new);
}

#[test]
fn caller_cap_is_inclusive_and_refused_glob_name_excludes_only_that_path() {
    // Joins caller cap -> literal refused pathspec -> real git tree, including a near-match name.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "at-cap", b"1234567");
    write(&f.repo.path, "large[1].dat", b"12345678");
    write(&f.repo.path, "large1.dat", b"keep");
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], 7).unwrap();
    assert!(saved.written);
    assert_eq!(saved.refused, vec![RefusedFile { path: PathBuf::from("large[1].dat"), size: 8, cap: 7 }]);
    assert_eq!(tree_names(&f.git, &saved.tree), ["at-cap", "large1.dat", "root.txt"]);
    assert_eq!(blob(&f.git, &saved.tree, "at-cap"), b"1234567");
    assert_eq!(blob(&f.git, &saved.tree, "large1.dat"), b"keep");
    assert_eq!(fs::read(f.repo.path.join("large[1].dat")).unwrap(), b"12345678");
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], 7).unwrap());
}

#[test]
fn new_against_head_file_that_grows_or_is_renamed_over_cap_drops_from_next_snapshot() {
    // Joins snapshot history -> HEAD-based cap membership -> real tree removal and repeat reporting (S-31).
    let f = Fixture::new(&[]);
    write(&f.repo.path, "new.txt", b"abc");
    let first = snapshot(&f.git, &f.repo.path, &f.target, &[], 3).unwrap();
    assert!(first.refused.is_empty());
    assert_eq!(blob(&f.git, &first.tree, "new.txt"), b"abc");
    write(&f.repo.path, "new.txt", b"abcd");
    let second = snapshot(&f.git, &f.repo.path, &f.target, &[], 3).unwrap();
    assert!(second.written);
    assert_eq!(second.refused, vec![RefusedFile { path: PathBuf::from("new.txt"), size: 4, cap: 3 }]);
    assert_eq!(tree_names(&f.git, &second.tree), ["root.txt"]);
    assert_eq!(f.git.commit_info(&second.commit).unwrap().parents, vec![f.head.clone(), first.commit]);
    fs::rename(f.repo.path.join("new.txt"), f.repo.path.join("renamed.txt")).unwrap();
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], 3).unwrap());
    let before = fs::read(oplog_path(&f.common)).unwrap();
    let third = snapshot(&f.git, &f.repo.path, &f.target, &[], 3).unwrap();
    assert!(!third.written);
    assert_eq!(third.commit, second.commit);
    assert_eq!(third.refused, vec![RefusedFile { path: PathBuf::from("renamed.txt"), size: 4, cap: 3 }]);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before);
    assert_eq!(fs::read(f.repo.path.join("renamed.txt")).unwrap(), b"abcd");
}
