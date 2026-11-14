use crate::support::Fixture;
use oplog::{
    acquire_lock, decode_lock_record, host_name, release_lock, AcquiredLock,
    KillPidProbe, LockError, LockName, LockRecord, PidProbe,
};
use std::fs;
use std::process::Command;
use testkit::test_child;
use std::sync::Barrier;

#[test]
fn acquire_fresh_directory_refuses_live_holder_then_release_removes_file() {
    // Joins host_name -> acquire_lock -> real exclusive-create/read -> release_lock.
    let f = Fixture::new();
    let me = holder("live-holder");
    let other = holder("contender");
    assert!(KillPidProbe.pid_alive(me.pid));
    assert!(!f.common.join("mortar").exists());
    let lock = acquire_lock(&f.common, &LockName::Land, &me, &KillPidProbe).unwrap();
    assert_eq!(lock.path, f.common.join("mortar/locks/land.lock"));
    assert_eq!(lock.record, me);
    assert_eq!(lock.took_over, None);
    let bytes = fs::read(&lock.path).unwrap();
    assert_eq!(decode_lock_record(std::str::from_utf8(&bytes).unwrap()).unwrap(), me);
    let error = acquire_lock(&f.common, &LockName::Land, &other, &KillPidProbe).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("live-holder") && message.contains(&me.host) && message.contains("since 100"), "{message}");
    match error {
        LockError::Held { path, holder } => {
            assert_eq!(path, lock.path);
            assert_eq!(holder, me);
        }
        other => panic!("expected Held: {other:?}"),
    }
    assert_eq!(fs::read(&lock.path).unwrap(), bytes);
    release_lock(&lock).unwrap();
    assert!(!lock.path.exists());
    assert!(matches!(release_lock(&lock), Err(LockError::NotOurs { holder: None, .. })));
    let again = acquire_lock(&f.common, &LockName::Land, &other, &KillPidProbe).unwrap();
    assert_eq!(again.record, other);
    release_lock(&again).unwrap();
}

#[test]
fn folders_lock_is_exclusive_per_store_and_distinct_from_join_and_land() {
    // Joins LockName::Folders -> lock_file_name -> acquire_lock's real publication -> release_lock.
    let f = Fixture::new();
    let me = holder("folders-holder");
    let contender = holder("folders-contender");
    let folders = acquire_lock(&f.common, &LockName::Folders, &me, &KillPidProbe).unwrap();
    assert_eq!(folders.path, f.common.join("mortar/locks/folders.lock"));
    assert_eq!(folders.record, me);
    assert_eq!(folders.took_over, None);
    let bytes = fs::read(&folders.path).unwrap();
    assert_eq!(decode_lock_record(std::str::from_utf8(&bytes).unwrap()).unwrap(), me);
    match acquire_lock(&f.common, &LockName::Folders, &contender, &KillPidProbe).unwrap_err() {
        LockError::Held { path, holder } => {
            assert_eq!(path, folders.path);
            assert_eq!(holder, me);
        }
        other => panic!("expected store-wide folders Held: {other:?}"),
    }
    assert_eq!(fs::read(&folders.path).unwrap(), bytes);
    let join = acquire_lock(&f.common, &LockName::Join("folders".into()), &me, &KillPidProbe).unwrap();
    let land = acquire_lock(&f.common, &LockName::Land, &me, &KillPidProbe).unwrap();
    assert_eq!(join.path, f.common.join("mortar/locks/join-folders.lock"));
    assert_eq!(land.path, f.common.join("mortar/locks/land.lock"));
    let other_store = Fixture::new();
    let independent = acquire_lock(&other_store.common, &LockName::Folders, &contender, &KillPidProbe).unwrap();
    assert_eq!(independent.path, other_store.common.join("mortar/locks/folders.lock"));
    release_lock(&folders).unwrap();
    assert!(!folders.path.exists());
    for lock in [&join, &land, &independent] {
        assert_eq!(decode_lock_record(&fs::read_to_string(&lock.path).unwrap()).unwrap(), lock.record);
    }
    let next = acquire_lock(&f.common, &LockName::Folders, &contender, &KillPidProbe).unwrap();
    assert_eq!(next.record, contender);
    assert_eq!(decode_lock_record(&fs::read_to_string(&next.path).unwrap()).unwrap(), contender);
    for lock in [&next, &join, &land, &independent] {
        release_lock(lock).unwrap();
        assert!(!lock.path.exists());
    }
}

#[test]
fn dead_process_lock_is_taken_over_and_previous_holder_cannot_release_it() {
    // Joins real process exit -> KillPidProbe -> acquire_lock's stale takeover -> release_lock ownership.
    let f = Fixture::new();
    let child = test_child("locks::dead_holder_worker").unwrap()
        .env("MORTAR_TEST_LOCK_REPO", &f.repo.path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn().unwrap();
    let dead_pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    assert!(!KillPidProbe.pid_alive(dead_pid));
    let previous = LockRecord { pid: dead_pid, key: "dead-holder".into(), ..holder("unused") };
    let me = holder("new-holder");
    let taken = acquire_lock(&f.common, &LockName::Land, &me, &KillPidProbe).unwrap();
    assert_eq!(taken.took_over, Some(previous.clone()));
    assert_eq!(taken.record, me);
    assert_eq!(decode_lock_record(&fs::read_to_string(&taken.path).unwrap()).unwrap(), me);
    assert!(!f.common.join("mortar/locks/land.lock.takeover").exists());
    assert_eq!(fs::read_dir(f.common.join("mortar/locks")).unwrap().count(), 1);
    let stale_handle = AcquiredLock { path: taken.path.clone(), record: previous, took_over: None };
    match release_lock(&stale_handle).unwrap_err() {
        LockError::NotOurs { path, holder: Some(current) } => {
            assert_eq!(path, taken.path);
            assert_eq!(current, me);
        }
        other => panic!("expected NotOurs with new holder: {other:?}"),
    }
    assert!(taken.path.is_file());
    release_lock(&taken).unwrap();
    assert_eq!(fs::read_dir(f.common.join("mortar/locks")).unwrap().count(), 0);
}

#[test]
fn two_racing_takers_have_exactly_one_winner() {
    // Joins concurrent acquire_lock calls -> real exclusive publication -> held refusal.
    // A barrier selects concurrency, not a timing threshold; winner stays held until both finish.
    let f = Fixture::new();
    let one = holder("racer-one");
    let two = holder("racer-two");
    let barrier = Barrier::new(2);
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            acquire_lock(&f.common, &LockName::Join("race".into()), &one, &KillPidProbe)
        });
        let second = scope.spawn(|| {
            barrier.wait();
            acquire_lock(&f.common, &LockName::Join("race".into()), &two, &KillPidProbe)
        });
        [first.join().unwrap(), second.join().unwrap()]
    });
    let mut winners = Vec::new();
    let mut refused = Vec::new();
    for result in results {
        match result {
            Ok(lock) => winners.push(lock),
            Err(LockError::Held { holder, .. }) => refused.push(holder),
            other => panic!("unexpected racing outcome: {other:?}"),
        }
    }
    assert_eq!(winners.len(), 1);
    assert_eq!(refused, vec![winners[0].record.clone()]);
    assert_eq!(winners[0].path, f.common.join("mortar/locks/join-race.lock"));
    release_lock(&winners[0]).unwrap();
}

#[test]
fn other_host_holder_is_not_stale_even_with_a_dead_pid() {
    // Joins a real lock file -> host-sensitive staleness decision -> held refusal.
    let f = Fixture::new();
    let mut exited = Command::new("true").spawn().unwrap();
    let dead_pid = exited.id();
    assert!(exited.wait().unwrap().success());
    assert!(!KillPidProbe.pid_alive(dead_pid));
    let remote = LockRecord {
        pid: dead_pid,
        host: format!("other-{}", host_name().unwrap()),
        ..holder("remote")
    };
    let lock = acquire_lock(&f.common, &LockName::Land, &remote, &KillPidProbe).unwrap();
    match acquire_lock(&f.common, &LockName::Land, &holder("local"), &KillPidProbe).unwrap_err() {
        LockError::Held { holder, .. } => assert_eq!(holder, remote),
        other => panic!("expected foreign Held: {other:?}"),
    }
    release_lock(&lock).unwrap();
}

fn holder(key: &str) -> LockRecord {
    LockRecord { pid: std::process::id(), host: host_name().unwrap(), key: key.into(), since: 100 }
}

#[test]
#[ignore = "subprocess entry point; parent waits for actual process exit before takeover"]
fn dead_holder_worker() {
    let repo = std::path::PathBuf::from(std::env::var_os("MORTAR_TEST_LOCK_REPO").unwrap());
    acquire_lock(&repo.join(".git"), &LockName::Land, &holder("dead-holder"), &KillPidProbe).unwrap();
    // AcquiredLock has no implicit release: exiting leaves the on-disk record for the parent.
}
