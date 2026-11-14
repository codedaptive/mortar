use crate::support::{run, setup, Fixture};
use folders::{make_request_folder, make_zone_folder, FolderError};
use gitio::Git;
use oplog::{acquire_lock, host_name, oplog_path, release_lock, KillPidProbe, LockError, LockName, LockRecord};
use std::fs;

#[test]
fn caller_floor_above_real_free_disk_refuses_before_branch_or_worktree_creation() {
    // Joins make -> actual DfFreeDisk OS probe -> floor refusal/sweep diagnostic -> no git/oplog writes.
    for request in [false, true] {
        let f = Fixture::new();
        if request { run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]); }
        let path = f.repo.path.join("not-made");
        let mut s = setup(&f.git, &path);
        s.floor_bytes = u64::MAX;
        let error = if request { make_request_folder(&s, "red", "one", &["kit/zone".into()]) }
            else { make_zone_folder(&s, "red", &["kit/zone".into()], Some(&f.head)) }.unwrap_err();
        let text = error.to_string();
        assert!(text.contains("1234") && text.contains("fixture sweep estimate"), "{text}");
        match error {
            FolderError::BelowFloor(below) => {
                assert_eq!(below.floor, u64::MAX);
                assert!(below.free < below.floor);
                assert_eq!(below.sweep.bytes, 1234);
                assert_eq!(below.sweep.reason.as_deref(), Some("fixture sweep estimate"));
            }
            other => panic!("expected BelowFloor from real probe: {other:?}"),
        }
        assert!(!path.exists());
        assert_eq!(f.git.worktree_list().unwrap().len(), 1);
        assert_eq!(f.git.read_ref("refs/heads/req/red/one").unwrap(), None);
        assert_eq!(f.git.read_ref("refs/heads/zone/red").unwrap(), if request { Some(f.head.clone()) } else { None });
        assert!(!oplog_path(&f.common).exists());
        assert!(!f.common.join("mortar/locks/folders.lock").exists());
    }
}

#[test]
fn live_folders_holder_refuses_both_makes_without_touching_holder_or_creating_anything() {
    // Joins make -> oplog acquire_lock/KillPidProbe -> retained live holder and no work side effects.
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let holder = LockRecord { pid: std::process::id(), host: host_name().unwrap(), key: "other-command".into(), since: 123 };
    let held = acquire_lock(&f.common, &LockName::Folders, &holder, &KillPidProbe).unwrap();
    let before = fs::read(&held.path).unwrap();
    let zone = f.repo.path.join("not-zone");
    let req = f.repo.path.join("not-request");
    let first = make_zone_folder(&setup(&f.git, &zone), "blocked", &["kit/zone".into()], Some(&f.head)).unwrap_err();
    let second = make_request_folder(&setup(&f.git, &req), "red", "one", &["kit/zone".into()]).unwrap_err();
    for error in [first, second] {
        let text = error.to_string();
        assert!(text.contains("other-command"), "{text}");
        match error {
            FolderError::Lock(LockError::Held { path, holder: actual, .. }) => { assert_eq!(path, held.path); assert_eq!(actual, holder); }
            other => panic!("expected held folders lock: {other:?}"),
        }
    }
    assert_eq!(fs::read(&held.path).unwrap(), before);
    assert!(!zone.exists() && !req.exists());
    assert_eq!(f.git.read_ref("refs/heads/zone/blocked").unwrap(), None);
    assert_eq!(f.git.read_ref("refs/heads/req/red/one").unwrap(), None);
    assert_eq!(f.git.worktree_list().unwrap().len(), 1);
    assert!(!oplog_path(&f.common).exists());
    release_lock(&held).unwrap();
    make_zone_folder(&setup(&f.git, &zone), "blocked", &["kit/zone".into()], Some(&f.head)).unwrap();
    assert_eq!(f.git.read_ref("refs/heads/zone/blocked").unwrap(), Some(f.head.clone()));
    assert!(!held.path.exists());
}

#[test]
fn branch_already_checked_out_elsewhere_refuses_naming_its_worktree_without_a_new_op() {
    // Joins make's worktree list -> branch holder guard -> preservation of an ordinary existing worktree.
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let held = f.repo.path.join("held-worktree");
    f.git.worktree_add(&held, &f.head, None).unwrap();
    let local = Git { repo: held.clone(), ..f.git.clone() };
    run(&local, &["checkout", "zone/red"]);
    let path = f.repo.path.join("not-made");
    let error = make_zone_folder(&setup(&f.git, &path), "red", &["kit/zone".into()], None).unwrap_err();
    match error {
        FolderError::BranchCheckedOut { branch, path: actual } => {
            assert_eq!(branch, "zone/red");
            assert_eq!(fs::canonicalize(actual).unwrap(), fs::canonicalize(&held).unwrap());
        }
        other => panic!("expected checked-out branch: {other:?}"),
    }
    assert!(!path.exists());
    assert_eq!(run(&local, &["status", "--porcelain"]), b"");
    assert_eq!(f.git.worktree_list().unwrap().len(), 2);
    assert!(!oplog_path(&f.common).exists());
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
}
