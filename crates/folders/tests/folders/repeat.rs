use crate::support::{assert_ordinary, run, setup, write, Fixture};
use folders::{make_request_folder, make_zone_folder, FolderError};
use folders::make::read_folder_record;
use folders::record::record_path;
use gitio::{Git, RunOpts};
use oplog::oplog_path;
use std::fs;
use testkit::write_git_shim;

#[test]
fn repeats_finish_real_interrupted_zone_and_request_fill_without_new_branch_ops() {
    // Joins make -> real worktree/record + interrupted sparse-set -> repeat refill -> original oplog bytes.
    for request in [false, true] {
        let f = Fixture::new();
        if request { run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]); }
        let path = f.repo.path.join("folder");
        let shim = f.common.join("interrupt-fill.sh");
        let marker = f.common.join("fill-interrupted");
        write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = sparse-checkout ] && [ \"$4\" = set ]; then\n  test -s \"$MORTAR_TEST_FOLDER_LOCK\"\n  directory=$(git -C \"$2\" rev-parse --absolute-git-dir)\n  test -s \"$directory/mortar-folder.json\"\n  printf 'record-before-fill\\n' >\"$MORTAR_TEST_FILL_MARKER\"\n  exit 75\nfi\nexec git \"$@\"\n").unwrap();
        let mut interrupted = f.git.clone();
        interrupted.program = shim;
        interrupted.env.extend([
            ("MORTAR_TEST_FOLDER_LOCK".into(), f.common.join("mortar/locks/folders.lock").into_os_string()),
            ("MORTAR_TEST_FILL_MARKER".into(), marker.clone().into_os_string()),
        ]);
        let paths = ["kit/zone".into()];
        let s = setup(&interrupted, &path);
        let result = if request { make_request_folder(&s, "red", "one", &paths) }
            else { make_zone_folder(&s, "red", &paths, Some(&f.head)) };
        assert!(matches!(result, Err(FolderError::Git(_))), "{result:?}");
        assert_eq!(fs::read(marker).unwrap(), b"record-before-fill\n");
        let git_dir = f.git.git_dir(&path).unwrap();
        let before_record = read_folder_record(&git_dir).unwrap().unwrap();
        let record_bytes = fs::read(record_path(&git_dir)).unwrap();
        assert!(!path.join("kit/zone/file").exists());
        assert!(!f.common.join("mortar/locks/folders.lock").exists());
        let before_log = fs::read(oplog_path(&f.common)).unwrap();
        assert_eq!(oplog::read_op_log(&f.common).unwrap().records.len(), 2);
        let s = setup(&f.git, &path);
        let record = if request { make_request_folder(&s, "red", "one", &paths).unwrap() }
            else { make_zone_folder(&s, "red", &paths, Some(&f.head)).unwrap() };
        assert_eq!(record, before_record);
        assert_eq!(fs::read(record_path(&git_dir)).unwrap(), record_bytes);
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
        assert_eq!(f.git.worktree_list().unwrap().len(), 2);
        assert_ordinary(&f.git, &path, if request { "req/red/one" } else { "zone/red" }, &f.head, &["kit/zone"]);
        assert_eq!(fs::read(path.join("kit/zone/file")).unwrap(), b"zone\n");
        write(&path, "kit/zone/file", b"unsaved work\n");
        write(&path, "kit/zone/untracked", b"untracked work\n");
        let again = if request { make_request_folder(&s, "red", "one", &paths).unwrap() }
            else { make_zone_folder(&s, "red", &paths, None).unwrap() };
        assert_eq!(again, record);
        assert_eq!(fs::read(path.join("kit/zone/file")).unwrap(), b"unsaved work\n");
        assert_eq!(fs::read(path.join("kit/zone/untracked")).unwrap(), b"untracked work\n");
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
    }
}

#[test]
fn different_record_refuses_naming_existing_folder_without_replacing_record_or_work() {
    // Joins repeat_decision -> DifferentFolder diagnostic -> real record/ref/log/file immutability.
    for request in [false, true] {
        let f = Fixture::new();
        if request { run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]); }
        let path = f.repo.path.join("folder");
        let s = setup(&f.git, &path);
        let record = if request { make_request_folder(&s, "red", "one", &["kit/zone".into()]).unwrap() }
            else { make_zone_folder(&s, "red", &["kit/zone".into()], Some(&f.head)).unwrap() };
        write(&path, "kit/zone/file", b"do not replace\n");
        let git_dir = f.git.git_dir(&path).unwrap();
        let before_record = fs::read(record_path(&git_dir)).unwrap();
        let before_log = fs::read(oplog_path(&f.common)).unwrap();
        let result = if request { make_request_folder(&s, "red", "two", &["kit/zone".into()]) }
            else { make_zone_folder(&s, "red", &["needed".into()], Some(&f.head)) };
        let error = result.unwrap_err();
        let text = error.to_string();
        assert!(text.contains(&path.display().to_string()) && text.contains("red"), "{text}");
        if request { assert!(text.contains("one"), "{text}"); }
        match error {
            FolderError::DifferentFolder { path: actual, existing } => { assert_eq!(actual, path); assert_eq!(existing, record); }
            other => panic!("expected DifferentFolder: {other:?}"),
        }
        assert_eq!(fs::read(record_path(&git_dir)).unwrap(), before_record);
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
        assert_eq!(fs::read(path.join("kit/zone/file")).unwrap(), b"do not replace\n");
        assert_eq!(f.git.read_ref("refs/heads/req/red/two").unwrap(), None);
        assert_eq!(f.git.worktree_list().unwrap().len(), 2);
        assert!(!f.common.join("mortar/locks/folders.lock").exists());
    }
}

#[test]
fn same_record_repeat_during_conflicted_rebase_preserves_unmerged_index_and_work() {
    // Joins request make/refill -> real git rebase conflict -> preservation of replay state.
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let path = f.repo.path.join("folder");
    let s = setup(&f.git, &path);
    let record = make_request_folder(&s, "red", "one", &["kit/zone".into()]).unwrap();
    let local = Git { repo: path.clone(), ..f.git.clone() };
    write(&path, "kit/zone/conflict", b"request side\n");
    run(&local, &["add", "kit/zone/conflict"]);
    run(&local, &["commit", "-m", "request change"]);
    write(&f.repo.path, "kit/zone/conflict", b"onto side\n");
    run(&f.git, &["add", "kit/zone/conflict"]);
    run(&f.git, &["commit", "-m", "onto change"]);
    let onto = f.git.rev_parse("HEAD").unwrap().unwrap();
    let out = local.run(&["rebase", "--onto", onto.as_str(), f.head.as_str()], &RunOpts { answer_codes: vec![1], ..RunOpts::default() }).unwrap();
    assert_eq!(out.status, 1);
    let unmerged = run(&local, &["ls-files", "--unmerged", "-z"]);
    assert!(!unmerged.is_empty());
    let git_dir = f.git.git_dir(&path).unwrap();
    assert!(git_dir.join("rebase-merge").exists());
    let index_before = fs::read(git_dir.join("index")).unwrap();
    let conflict_before = fs::read(path.join("kit/zone/conflict")).unwrap();
    let log_before = fs::read(oplog_path(&f.common)).unwrap();
    let result = make_request_folder(&s, "red", "one", &["kit/zone".into()]);
    assert_eq!(run(&local, &["ls-files", "--unmerged", "-z"]), unmerged);
    assert_eq!(fs::read(git_dir.join("index")).unwrap(), index_before);
    assert_eq!(fs::read(path.join("kit/zone/conflict")).unwrap(), conflict_before);
    assert!(git_dir.join("rebase-merge").exists());
    assert_eq!(read_folder_record(&git_dir).unwrap(), Some(record.clone()));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log_before);
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
    assert_eq!(result.unwrap(), record, "same record is returned without refusal");
}
