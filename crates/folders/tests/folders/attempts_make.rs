//! make_attempts in a throwaway repository: the cases src/attempts/make_attempts.rs's header names (S-69).
use super::attempt_support::{make, try_ref, Attempts, REQ_REF};
use super::maintenance_support::records;
use crate::support::{assert_branch_op, assert_ordinary, Fixture};
use folders::make::read_folder_record;
use folders::{FolderError, FolderKind, FolderRecord};
use oplog::{acquire_lock, host_name, oplog_path, release_lock, KillPidProbe, LockError, LockName, LockRecord};
use std::fs;

fn expected(a: &Attempts, k: u32) -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Attempt(k), zone: "red".into(), request: Some("one".into()), base: a.f.head.clone(),
        work_order_paths: vec!["kit/zone".into()], needed_paths: vec![], parent: Some("req/red/one".into()),
    }
}

#[test]
fn three_attempts_sit_beside_the_request_branch_each_on_its_try_branch_at_the_request_tip_with_one_op_each() {
    let a = Attempts::new();
    let start = records(&a.f).len();
    let made = a.make(3).unwrap();
    assert_eq!(made, vec![expected(&a, 1), expected(&a, 2), expected(&a, 3)]);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(a.f.head.clone()));
    for k in 1..=3 {
        let folder = a.dir.join(format!("try-{k}"));
        assert_eq!(a.f.git.read_ref(&try_ref(k)).unwrap(), Some(a.f.head.clone()));
        assert_ordinary(&a.f.git, &folder, &format!("try/red/one/{k}"), &a.f.head, &["kit/zone"]);
        assert_eq!(read_folder_record(&a.f.git.git_dir(&folder).unwrap()).unwrap(), Some(expected(&a, k)));
    }
    let ops = records(&a.f);
    assert_eq!(ops.len(), start + 6);
    for k in 1..=3u32 {
        let at = start + 2 * (k as usize - 1);
        assert_branch_op(&ops[at..at + 2], &format!("try/red/one/{k}"), &a.f.head);
    }
}

#[test]
fn a_repeat_makes_nothing_new_and_returns_the_same_records() {
    let a = Attempts::with(2);
    let before = a.state();
    let again = a.make(2).unwrap();
    assert_eq!(again, vec![expected(&a, 1), expected(&a, 2)]);
    assert_eq!(a.state(), before);
}

#[test]
fn zero_attempts_is_plan_with_nothing_made() {
    let a = Attempts::new();
    let before = a.state();
    match a.make(0) {
        Err(FolderError::Plan(_)) => {}
        other => panic!("expected Plan, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert!(!a.dir.exists());
}

#[test]
fn a_missing_request_branch_is_request_branch_missing_with_nothing_made() {
    let f = Fixture::new();
    let dir = f.repo.path.join("attempts");
    match make(&f, &dir, 2) {
        Err(FolderError::RequestBranchMissing { branch }) => assert_eq!(branch, "req/red/one"),
        other => panic!("expected RequestBranchMissing, got {other:?}"),
    }
    assert!(!dir.exists());
    assert_eq!(f.git.read_ref(&try_ref(1)).unwrap(), None);
    assert_eq!(f.git.worktree_list().unwrap().len(), 1);
    assert!(!oplog_path(&f.common).exists());
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
}

#[test]
fn a_folders_lock_held_by_a_live_pid_is_lock_with_nothing_made_and_the_holder_untouched() {
    let a = Attempts::new();
    let holder = LockRecord { pid: std::process::id(), host: host_name().unwrap(), key: "other-command".into(), since: 123 };
    let held = acquire_lock(&a.f.common, &LockName::Folders, &holder, &KillPidProbe).unwrap();
    let lock = fs::read(&held.path).unwrap();
    let before = a.state();
    match a.make(2) {
        Err(FolderError::Lock(LockError::Held { path, holder: actual, .. })) => {
            assert_eq!(path, held.path);
            assert_eq!(actual, holder);
        }
        other => panic!("expected Lock, got {other:?}"),
    }
    assert_eq!(fs::read(&held.path).unwrap(), lock);
    assert_eq!(a.state(), before);
    assert!(!a.dir.exists());
    release_lock(&held).unwrap();
}
