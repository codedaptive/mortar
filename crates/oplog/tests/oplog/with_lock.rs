//! Public with_lock joins real PID probes, lock files, oplog durability and work; each takeover
//! guard the acquire removed, on success or refusal, is logged as an op.
use crate::support::{refs, Fixture};
use oplog::{acquire_lock, decode_lock_record, host_name, oplog_path, read_op_log,
    release_lock, with_lock, HeldLockError, KillPidProbe, LockError, LockName, LockRecord, OpPhase, OpResult, PidProbe};
use std::{cell::Cell, fs, path::{Path, PathBuf}, process::Command, sync::atomic::{AtomicBool, Ordering}};

#[test]
fn work_observes_the_named_lock_and_success_or_work_error_always_releases_it() {
    // Joins with_lock -> actual acquire -> closure observation/value/error -> actual release, all lock namespaces.
    for (name, filename) in [(LockName::Land, "land.lock"), (LockName::Folders, "folders.lock"),
        (LockName::Join("sample".into()), "join-sample.lock")] {
        for fail in [false, true] {
            let f = Fixture::new();
            let path = f.common.join("mortar/locks").join(filename);
            let before = refs(&f);
            let called = Cell::new(0);
            let result = with_lock(&f.git, &f.common, &name, "closure-key", &KillPidProbe, || {
                called.set(called.get() + 1);
                let holder = decode_lock_record(&fs::read_to_string(&path).unwrap()).unwrap();
                assert_eq!(holder.pid, std::process::id());
                assert_eq!(holder.host, host_name().unwrap());
                assert_eq!(holder.key, "closure-key");
                let other = record(std::process::id(), "competing-reader");
                assert!(matches!(acquire_lock(&f.common, &name, &other, &KillPidProbe), Err(LockError::Held { .. })));
                if fail { Err("exact work error") } else { Ok("exact work value") }
            });
            if fail { assert!(matches!(result, Err(HeldLockError::Work("exact work error")))); }
            else { assert_eq!(result.unwrap(), "exact work value"); }
            assert_eq!(called.get(), 1);
            assert!(!path.exists());
            assert!(!oplog_path(&f.common).exists(), "fresh lock requires no takeover/guard op");
            assert_eq!(refs(&f), before);
            let after = acquire_lock(&f.common, &name, &record(std::process::id(), "reacquire"), &KillPidProbe).unwrap();
            release_lock(&after).unwrap();
        }
    }
}

#[test]
fn stale_holder_takeover_is_durable_before_work_and_moves_no_ref() {
    // Joins real child death -> stale lock acquire -> durable takeover op -> closure entry and release.
    let f = Fixture::new();
    let stale = record(dead_pid(), "dead-holder");
    let path = f.common.join("mortar/locks/land.lock");
    put(&path, &stale);
    let before = refs(&f);
    with_lock(&f.git, &f.common, &LockName::Land, "take-over", &KillPidProbe, || {
        let now = decode_lock_record(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(now.key, "take-over");
        assert_eq!(now.pid, std::process::id());
        let records = read_op_log(&f.common).unwrap().records;
        assert_pair(&records, "lock-takeover", &format!("lock-takeover:{}:{}@{}:{}", path.display(), stale.pid, stale.host, stale.since));
        for detail in [stale.pid.to_string(), stale.host.clone(), stale.key.clone()] { assert!(records[1].reason.contains(&detail)); }
        assert_eq!(refs(&f), before);
        Ok::<_, ()>(())
    }).unwrap();
    assert!(!path.exists());
    assert_eq!(read_op_log(&f.common).unwrap().records.len(), 2);
    assert_eq!(refs(&f), before);
}

#[test]
fn live_holder_refuses_as_lock_without_work_or_releasing_someone_elses_file() {
    // Joins exclusive actual lock -> with_lock refusal mapping -> no closure/log/ref/file mutation.
    let f = Fixture::new();
    let held = acquire_lock(&f.common, &LockName::Land, &record(std::process::id(), "existing-owner"), &KillPidProbe).unwrap();
    let bytes = fs::read(&held.path).unwrap();
    let before = refs(&f);
    let ran = Cell::new(false);
    let error = with_lock(&f.git, &f.common, &LockName::Land, "must-not-run", &KillPidProbe, || {
        ran.set(true); Ok::<_, ()>(())
    }).unwrap_err();
    match error {
        HeldLockError::Lock(LockError::Held { path, holder, .. }) => { assert_eq!(path, held.path); assert_eq!(holder, held.record); }
        other => panic!("expected held-lock error, got {other:?}"),
    }
    assert!(!ran.get());
    assert_eq!(fs::read(&held.path).unwrap(), bytes);
    assert!(!oplog_path(&f.common).exists());
    assert_eq!(refs(&f), before);
    release_lock(&held).unwrap();
}

#[test]
fn removed_guard_is_logged_before_takeover_and_work_when_the_acquire_wins() {
    // Joins actual guard reaping -> automatic guard op -> takeover op -> work, with durable ordering.
    let (f, path, guard_path, stale, guard) = guard_fixture();
    let before = refs(&f);
    with_lock(&f.git, &f.common, &LockName::Land, "guard-winner", &KillPidProbe, || {
        assert!(!guard_path.exists());
        let log = read_op_log(&f.common).unwrap();
        assert!(log.torn_lines.is_empty());
        assert_eq!(log.records.len(), 4);
        assert_pair(&log.records[..2], "takeover-guard-removal", &guard_key(&guard_path, &guard));
        assert_pair(&log.records[2..], "lock-takeover", &format!("lock-takeover:{}:{}@{}:{}", path.display(), stale.pid, stale.host, stale.since));
        for detail in [guard.pid.to_string(), guard.host.clone(), guard.key.clone()] { assert!(log.records[1].reason.contains(&detail)); }
        assert_eq!(decode_lock_record(&fs::read_to_string(&path).unwrap()).unwrap().key, "guard-winner");
        Ok::<_, ()>(())
    }).unwrap();
    assert!(!path.exists());
    assert!(!guard_path.exists());
    assert_eq!(read_op_log(&f.common).unwrap().records.len(), 4);
    assert_eq!(refs(&f), before);
}

#[test]
fn removed_guard_is_still_logged_when_a_real_competing_holder_makes_acquire_refuse() {
    // Inject only a deterministic filesystem event; every liveness answer still comes from the kernel.
    let (f, path, guard_path, _, guard) = guard_fixture();
    let replacement = record(std::process::id(), "competing-live-owner");
    let probe = ReplacingProbe { guard_pid: guard.pid, path: path.clone(), replacement: replacement.clone(), replaced: AtomicBool::new(false) };
    let before = refs(&f);
    let ran = Cell::new(false);
    let error = with_lock(&f.git, &f.common, &LockName::Land, "guard-loser", &probe, || {
        ran.set(true); Ok::<_, ()>(())
    }).unwrap_err();
    match error {
        HeldLockError::Lock(LockError::Held { path: actual, holder, .. }) => { assert_eq!(actual, path); assert_eq!(holder, replacement); }
        other => panic!("expected refusal after guard removal, got {other:?}"),
    }
    assert!(probe.replaced.load(Ordering::SeqCst));
    assert!(!ran.get());
    assert!(!guard_path.exists());
    assert_eq!(decode_lock_record(&fs::read_to_string(&path).unwrap()).unwrap(), replacement);
    let records = read_op_log(&f.common).unwrap().records;
    assert_pair(&records, "takeover-guard-removal", &guard_key(&guard_path, &guard));
    assert!(records.iter().all(|r| r.kind != "lock-takeover"));
    assert_eq!(refs(&f), before);
}

fn record(pid: u32, key: &str) -> LockRecord { LockRecord { pid, host: host_name().unwrap(), key: key.into(), since: 100 } }
fn dead_pid() -> u32 {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id();
    assert!(child.wait().unwrap().success());
    assert!(!KillPidProbe.pid_alive(pid));
    pid
}
/// Writes `record` as lock file text: the four `name=value` lines pid, host, key, since.
fn put(path: &Path, record: &LockRecord) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("pid={}\nhost={}\nkey={}\nsince={}\n", record.pid, record.host, record.key, record.since)).unwrap();
}
fn guard_fixture() -> (Fixture, PathBuf, PathBuf, LockRecord, LockRecord) {
    let f = Fixture::new();
    let mut one = Command::new("true").spawn().unwrap();
    let mut two = Command::new("true").spawn().unwrap();
    let stale = record(one.id(), "stale-holder");
    let guard = record(two.id(), "stale-guard");
    assert_ne!(stale.pid, guard.pid);
    assert!(one.wait().unwrap().success()); assert!(two.wait().unwrap().success());
    assert!(!KillPidProbe.pid_alive(stale.pid)); assert!(!KillPidProbe.pid_alive(guard.pid));
    let path = f.common.join("mortar/locks/land.lock");
    let guard_path = f.common.join("mortar/locks/land.lock.takeover");
    put(&path, &stale); put(&guard_path, &guard);
    (f, path, guard_path, stale, guard)
}
fn guard_key(path: &Path, guard: &LockRecord) -> String { format!("takeover-guard-removal:{}:{}@{}:{}", path.display(), guard.pid, guard.host, guard.since) }
fn assert_pair(records: &[oplog::OpRecord], kind: &str, key: &str) {
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].phase, OpPhase::Intent); assert_eq!(records[1].phase, OpPhase::Done);
    assert_eq!(records[0].op_id, records[1].op_id);
    assert!(records.iter().all(|r| r.kind == kind && r.key == key && r.moves.is_empty()));
    assert_eq!(records[1].result, Some(OpResult::Moved));
    assert_eq!(records[1].actor, format!("{}@{}", std::process::id(), host_name().unwrap()));
}
struct ReplacingProbe { guard_pid: u32, path: PathBuf, replacement: LockRecord, replaced: AtomicBool }
impl PidProbe for ReplacingProbe {
    fn pid_alive(&self, pid: u32) -> bool {
        if pid == self.guard_pid && !self.replaced.swap(true, Ordering::SeqCst) { put(&self.path, &self.replacement); }
        KillPidProbe.pid_alive(pid)
    }
}
