//! WO-orc-snapshot-it-5: SPEC-snapshot WO-snapshot-1/-2, S-31; seven remaining I/O cases in WO-snapshot-4d.
//! All private sizing/tree decisions are exercised through snapshot/is_saved, real gitio and oplog, not source includes.
use crate::support::{assert_snapshot_written, blob, run, tree_names, write, Fixture};
use oplog::{oplog_path, read_op_log, OpPhase, OpResult};
use snapshot::{is_saved, snapshot, RefusedFile, SnapshotError, DEFAULT_SIZE_CAP};
use std::{fs, io::ErrorKind, os::unix::fs::symlink, path::PathBuf};
use testkit::write_git_shim;

#[test]
fn real_metadata_symlink_loop_is_file_size_error_before_commit_ref_or_log_and_retry_preserves_work() {
    // Joins snapshot -> real diff paths -> filesystem event -> file_size's actual ELOOP -> no oplog apply.
    let f = Fixture::new(&[("changing/file", b"old content\n")]);
    write(&f.repo.path, "changing/file", b"new preserved content\n");
    let index_before = fs::read(f.common.join("index")).unwrap();
    let shim = f.common.join("metadata-event.sh");
    let backup = f.common.join("preserved-directory");
    let marker = f.common.join("metadata-event-done");
    write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = diff ] && mkdir \"$MORTAR_SIZE_ONCE\" 2>/dev/null; then\n  git \"$@\"\n  mv \"$MORTAR_SIZE_DIRECTORY\" \"$MORTAR_SIZE_BACKUP\"\n  ln -s changing \"$MORTAR_SIZE_DIRECTORY\"\n  printf 'done\\n' >\"$MORTAR_SIZE_MARKER\"\n  exit 0\nfi\nexec git \"$@\"\n").unwrap();
    let mut git = f.git.clone();
    git.program = shim;
    git.env.extend([
        ("MORTAR_SIZE_ONCE".into(), f.common.join("metadata-once").into_os_string()),
        ("MORTAR_SIZE_DIRECTORY".into(), f.repo.path.join("changing").into_os_string()),
        ("MORTAR_SIZE_BACKUP".into(), backup.clone().into_os_string()),
        ("MORTAR_SIZE_MARKER".into(), marker.clone().into_os_string()),
    ]);
    match snapshot(&git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::FileSize { path, error } => {
            assert_eq!(path, PathBuf::from("changing/file"));
            assert_ne!(error.kind(), ErrorKind::NotFound);
            #[cfg(target_os = "macos")]
            assert_eq!(error.raw_os_error(), Some(62));
            #[cfg(target_os = "linux")]
            assert_eq!(error.raw_os_error(), Some(40));
        }
        other => panic!("expected genuine FileSize/ELOOP, got {other:?}"),
    }
    assert_eq!(fs::read(marker).unwrap(), b"done\n");
    assert_eq!(fs::read(backup.join("file")).unwrap(), b"new preserved content\n");
    assert_eq!(fs::read_link(f.repo.path.join("changing")).unwrap(), PathBuf::from("changing"));
    assert_unwritten(&f);
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    // Restore only test-created paths, then show the original content remains saveable through the real pipeline.
    fs::remove_file(f.repo.path.join("changing")).unwrap();
    fs::rename(backup, f.repo.path.join("changing")).unwrap();
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(blob(&f.git, &saved.tree, "changing/file"), b"new preserved content\n");
}

#[test]
fn caller_noglob_environment_produces_exact_cap_mismatch_without_losing_previous_snapshot() {
    // Joins caller Git.env -> real git pathspec interpretation -> capped/full tree comparison -> CapMismatch.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "glob[1].bin", b"12345678");
    let first = snapshot(&f.git, &f.repo.path, &f.target, &[], 3).unwrap();
    assert_snapshot_written(&f, &first, &f.head);
    assert_eq!(first.refused, vec![RefusedFile { path: PathBuf::from("glob[1].bin"), size: 8, cap: 3 }]);
    assert_eq!(tree_names(&f.git, &first.tree), ["root.txt"]);
    let before_log = fs::read(oplog_path(&f.common)).unwrap();
    let before_index = fs::read(f.common.join("index")).unwrap();
    let before_commits = commit_ids(&f);
    let mut git = f.git.clone();
    // Deliberately caller-scoped, not process-global: WO-gitio-4 scrubs inherited pathspec variables.
    git.env.push(("GIT_NOGLOB_PATHSPECS".into(), "1".into()));
    for error in [
        snapshot(&git, &f.repo.path, &f.target, &[], 3).unwrap_err(),
        is_saved(&git, &f.repo.path, &f.target, &[], 3).unwrap_err(),
    ] {
        match error {
            SnapshotError::CapMismatch { paths } => assert_eq!(paths, vec![PathBuf::from("glob[1].bin")]),
            other => panic!("expected caller-env CapMismatch, got {other:?}"),
        }
    }
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
    assert_eq!(commit_ids(&f), before_commits);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(first.commit));
    assert_eq!(fs::read(f.repo.path.join("glob[1].bin")).unwrap(), b"12345678");
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], 3).unwrap());
}

#[test]
fn symlink_cap_uses_its_own_eleven_byte_length_not_sixty_four_byte_target() {
    // Joins file_size symlink_metadata (via snapshot) -> refused size -> exact-cap tree inclusion and logged move.
    let target_bytes = vec![b't'; 64];
    let f = Fixture::new(&[("target-file", &target_bytes)]);
    symlink("target-file", f.repo.path.join("link")).unwrap();
    let refused = snapshot(&f.git, &f.repo.path, &f.target, &[], 10).unwrap();
    assert_snapshot_written(&f, &refused, &f.head);
    assert_eq!(refused.refused, vec![RefusedFile { path: PathBuf::from("link"), size: 11, cap: 10 }]);
    assert_eq!(tree_names(&f.git, &refused.tree), ["root.txt", "target-file"]);
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], 11).unwrap();
    assert!(saved.written);
    assert!(saved.refused.is_empty());
    assert_eq!(tree_names(&f.git, &saved.tree), ["link", "root.txt", "target-file"]);
    assert_eq!(blob(&f.git, &saved.tree, "link"), b"target-file");
    assert!(run(&f.git, &["ls-tree", saved.tree.as_str(), "link"]).starts_with(b"120000 blob "));
    assert_eq!(blob(&f.git, &saved.tree, "target-file"), target_bytes);
    assert_eq!(fs::read_link(f.repo.path.join("link")).unwrap(), PathBuf::from("target-file"));
    assert_eq!(f.git.commit_info(&saved.commit).unwrap().parents, vec![f.head.clone(), refused.commit]);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(saved.commit));
    let log = read_op_log(&f.common).unwrap();
    assert_eq!(log.records.len(), 4);
    assert_eq!(log.records[3].phase, OpPhase::Done);
    assert_eq!(log.records[3].result, Some(OpResult::Moved));
}

#[test]
fn deleted_changed_path_is_not_a_size_refusal_even_at_zero_cap() {
    // Joins real deleted path from git diff -> metadata NotFound/None -> deletion in snapshot tree and oplog.
    let f = Fixture::new(&[("gone", b"remove this tracked file\n")]);
    fs::remove_file(f.repo.path.join("gone")).unwrap();
    let index_before = fs::read(f.common.join("index")).unwrap();
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], 0).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert!(saved.refused.is_empty());
    assert_eq!(tree_names(&f.git, &saved.tree), ["root.txt"]);
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"base\n");
    assert!(!f.repo.path.join("gone").exists());
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], 0).unwrap());
}

#[test]
fn new_gitlink_directory_is_not_treated_as_a_file_or_refused_at_zero_cap() {
    // Joins actual new embedded repository/gitlink -> directory metadata/None -> saved gitlink, not a capped file.
    let f = Fixture::new(&[]);
    let source = Fixture::new(&[("subfile", b"local child repository\n")]);
    // Local disposable source, no network or real-team repository, and no root index edit for fixture setup.
    run(&f.git, &["clone", "--local", "--no-hardlinks", source.repo.path.to_str().unwrap(), "module"]);
    assert!(f.repo.path.join("module").is_dir());
    let index_before = fs::read(f.common.join("index")).unwrap();
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], 0).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert!(saved.refused.is_empty());
    assert_eq!(tree_names(&f.git, &saved.tree), ["module", "root.txt"]);
    assert_eq!(run(&f.git, &["ls-tree", saved.tree.as_str(), "module"]), format!("160000 commit {}\tmodule\n", source.head).as_bytes());
    assert_eq!(fs::read(f.repo.path.join("module/subfile")).unwrap(), b"local child repository\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], 0).unwrap());
}

#[test]
fn unborn_head_is_named_before_any_snapshot_commit_ref_or_op() {
    // Joins folder_tree -> actual symbolic-but-unborn HEAD -> public UnbornHead error, without apply.
    let f = Fixture::new(&[]);
    run(&f.git, &["update-ref", "-d", "refs/heads/main", f.head.as_str()]);
    assert_eq!(run(&f.git, &["symbolic-ref", "HEAD"]), b"refs/heads/main\n");
    assert_eq!(f.git.read_ref("refs/heads/main").unwrap(), None);
    let before_index = fs::read(f.common.join("index")).unwrap();
    for error in [
        snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap_err(),
        is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap_err(),
    ] {
        match error {
            SnapshotError::UnbornHead { folder } => assert_eq!(folder, f.repo.path),
            other => panic!("expected UnbornHead, got {other:?}"),
        }
    }
    assert_unwritten(&f);
    assert_eq!(f.git.read_ref("refs/heads/main").unwrap(), None);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"base\n");
}

#[test]
fn over_cap_file_replacing_head_directory_is_refused_without_resurrecting_old_children() {
    // Joins HEAD path membership -> literal cap exclude -> real removal of old directory leaves -> logged snapshot.
    let f = Fixture::new(&[("replace/child", b"old child\n"), ("replace/nested/leaf", b"old nested leaf\n")]);
    fs::remove_file(f.repo.path.join("replace/child")).unwrap();
    fs::remove_file(f.repo.path.join("replace/nested/leaf")).unwrap();
    fs::remove_dir(f.repo.path.join("replace/nested")).unwrap();
    fs::remove_dir(f.repo.path.join("replace")).unwrap();
    write(&f.repo.path, "replace", b"1234");
    let index_before = fs::read(f.common.join("index")).unwrap();
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &[], 3).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(saved.refused, vec![RefusedFile { path: PathBuf::from("replace"), size: 4, cap: 3 }]);
    assert_eq!(tree_names(&f.git, &saved.tree), ["root.txt"]);
    assert_eq!(fs::read(f.repo.path.join("replace")).unwrap(), b"1234");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], 3).unwrap());
    let before_log = fs::read(oplog_path(&f.common)).unwrap();
    let again = snapshot(&f.git, &f.repo.path, &f.target, &[], 3).unwrap();
    assert!(!again.written);
    assert_eq!(again.commit, saved.commit);
    assert_eq!(again.refused, saved.refused);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
}

fn commit_ids(f: &Fixture) -> Vec<String> {
    let raw = run(&f.git, &["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]);
    let mut commits: Vec<_> = std::str::from_utf8(&raw).unwrap().lines()
        .filter_map(|line| line.strip_prefix("commit ").map(str::to_owned)).collect();
    commits.sort();
    commits
}

fn assert_unwritten(f: &Fixture) {
    assert!(!oplog_path(&f.common).exists());
    assert!(read_op_log(&f.common).unwrap().records.is_empty());
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), None);
    assert_eq!(commit_ids(f), vec![f.head.to_string()]);
}
