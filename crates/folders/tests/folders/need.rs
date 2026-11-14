use crate::support::{run, setup, Fixture};
use folders::{gaps, make_request_folder, need, FolderError};
use folders::make::read_folder_record;
use folders::record::record_path;
use gitio::Git;
use oplog::{oplog_path, KillPidProbe};
use std::fs;
use testkit::write_git_shim;

#[test]
fn need_widens_real_cone_and_record_repeat_is_inert_and_repeated_make_keeps_needed_paths() {
    // Joins need -> gitio sparse_add -> S-18 record -> gaps -> make/refill union of work-order and needed paths.
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let path = f.repo.path.join("folder");
    let s = setup(&f.git, &path);
    let made = make_request_folder(&s, "red", "one", &["kit/zone".into()]).unwrap();
    assert!(!path.join("needed").exists());
    assert!(gaps(&made).is_empty());
    let log_before = fs::read(oplog_path(&f.common)).unwrap();
    let covered = need(&f.git, &path, "kit/zone", &KillPidProbe).unwrap();
    assert_eq!(covered.needed_paths, ["kit/zone"]);
    assert!(gaps(&covered).is_empty());
    let record = need(&f.git, &path, "needed/", &KillPidProbe).unwrap();
    let mut expected = made;
    expected.needed_paths = vec!["kit/zone".into(), "needed".into()];
    assert_eq!(record, expected);
    assert_eq!(gaps(&record), ["needed"]);
    assert_eq!(fs::read(path.join("needed/file")).unwrap(), b"needed\n");
    let local = Git { repo: path.clone(), ..f.git.clone() };
    assert_eq!(run(&local, &["sparse-checkout", "list"]), b"kit/zone\nneeded\n");
    let git_dir = f.git.git_dir(&path).unwrap();
    assert_eq!(read_folder_record(&git_dir).unwrap(), Some(record.clone()));
    let record_before = fs::read(record_path(&git_dir)).unwrap();
    let index_before = fs::read(git_dir.join("index")).unwrap();
    assert_eq!(need(&f.git, &path, "needed", &KillPidProbe).unwrap(), record);
    assert_eq!(fs::read(record_path(&git_dir)).unwrap(), record_before);
    assert_eq!(fs::read(git_dir.join("index")).unwrap(), index_before);
    assert_eq!(make_request_folder(&s, "red", "one", &["kit/zone".into()]).unwrap(), record);
    assert_eq!(run(&local, &["sparse-checkout", "list"]), b"kit/zone\nneeded\n");
    assert_eq!(fs::read(path.join("needed/file")).unwrap(), b"needed\n");
    assert_eq!(fs::read(record_path(&git_dir)).unwrap(), record_before);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    assert_eq!(run(&local, &["status", "--porcelain"]), b"");
}

#[test]
fn repeated_need_finishes_interruption_after_record_write_before_sparse_add() {
    // Joins need's durable-first record write -> actual sparse-add refusal -> repeat completion, without extra ops.
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let path = f.repo.path.join("folder");
    make_request_folder(&setup(&f.git, &path), "red", "one", &["kit/zone".into()]).unwrap();
    let shim = f.common.join("interrupt-need.sh");
    write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = sparse-checkout ] && [ \"$4\" = add ]; then exit 75; fi\nexec git \"$@\"\n").unwrap();
    let interrupted = Git { program: shim, ..f.git.clone() };
    assert!(matches!(need(&interrupted, &path, "needed", &KillPidProbe), Err(FolderError::Git(_))));
    assert!(!path.join("needed").exists());
    let git_dir = f.git.git_dir(&path).unwrap();
    let record = read_folder_record(&git_dir).unwrap().unwrap();
    assert_eq!(record.needed_paths, ["needed"]);
    let bytes = fs::read(record_path(&git_dir)).unwrap();
    let log_before = fs::read(oplog_path(&f.common)).unwrap();
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
    assert_eq!(need(&f.git, &path, "needed", &KillPidProbe).unwrap(), record);
    assert_eq!(fs::read(path.join("needed/file")).unwrap(), b"needed\n");
    assert_eq!(fs::read(record_path(&git_dir)).unwrap(), bytes);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    assert!(!git_dir.join("mortar-folder.json.tmp").exists());
}

#[test]
fn need_refuses_plain_worktree_and_invalid_path_without_adopting_or_changing_record() {
    // Joins need's record/path validation -> untouched real worktree and record, with released lock.
    let f = Fixture::new();
    let plain = f.repo.path.join("plain");
    f.git.worktree_add(&plain, &f.head, None).unwrap();
    let error = need(&f.git, &plain, "needed", &KillPidProbe).unwrap_err();
    assert!(matches!(error, FolderError::NotAFolder { path, .. } if path == plain));
    assert!(!record_path(&f.git.git_dir(&plain).unwrap()).exists());
    assert!(!oplog_path(&f.common).exists());
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let path = f.repo.path.join("folder");
    make_request_folder(&setup(&f.git, &path), "red", "one", &["kit/zone".into()]).unwrap();
    let record_file = record_path(&f.git.git_dir(&path).unwrap());
    let before_record = fs::read(&record_file).unwrap();
    let before_log = fs::read(oplog_path(&f.common)).unwrap();
    assert!(matches!(need(&f.git, &path, "../outside", &KillPidProbe), Err(FolderError::Plan(_))));
    assert_eq!(fs::read(&record_file).unwrap(), before_record);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
}
