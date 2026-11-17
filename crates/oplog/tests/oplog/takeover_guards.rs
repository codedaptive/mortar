//! S-30: joins real dead/live PID evidence, on-disk takeover/reap guards, acquire results and caller-side op logging.

use super::locks::holder;
use crate::support::{refs, Fixture};
use oplog::{
    acquire_lock, apply, decode_lock_record, encode_lock_record, guard_removal_op, oplog_path,
    read_op_log, release_lock, ApplyOutcome, KillPidProbe, LockError, LockName, LockRecord, OpPhase, PidProbe,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Barrier};

#[test]
fn live_and_other_host_takeover_guards_are_refused_without_removal() {
    for remote in [false, true] {
        let c = guard_case();
        let guard = if remote {
            LockRecord { host: format!("other-{}", c.dead_guard.host), ..c.dead_guard.clone() }
        } else {
            holder("live-guard")
        };
        write_record(&c.guard, &guard);
        let before_lock = fs::read(&c.path).unwrap();
        let before_guard = fs::read(&c.guard).unwrap();
        let error = acquire_lock(&c.f.common, &LockName::Land, &c.me, &KillPidProbe).unwrap_err();
        assert!(error.removed_guards().is_empty());
        match error {
            LockError::TakeoverInProgress { path, holder, .. } => {
                assert_eq!(path, c.guard);
                assert_eq!(holder, guard);
            }
            other => panic!("expected live/remote guard refusal: {other:?}"),
        }
        assert_eq!(fs::read(&c.path).unwrap(), before_lock);
        assert_eq!(fs::read(&c.guard).unwrap(), before_guard);
        assert_eq!(fs::read_dir(c.path.parent().unwrap()).unwrap().count(), 2);
        assert!(!oplog_path(&c.f.common).exists());
    }
}

#[test]
fn dead_same_host_guard_is_removed_returned_and_logged_by_the_caller() {
    let c = guard_case();
    let before_refs = refs(&c.f);
    let acquired = acquire_lock(&c.f.common, &LockName::Land, &c.me, &KillPidProbe).unwrap();
    assert_eq!(acquired.path, c.path);
    assert_eq!(acquired.record, c.me);
    assert_eq!(acquired.took_over, Some(c.stale.clone()));
    assert_eq!(acquired.removed_guards, vec![c.dead_guard.clone()]);
    assert!(!c.guard.exists());
    assert!(!reap_path(&c).exists());
    assert_eq!(decode_lock_record(&fs::read_to_string(&c.path).unwrap()).unwrap(), c.me);
    assert_eq!(fs::read_dir(c.path.parent().unwrap()).unwrap().count(), 1);
    log_removals(&c, &acquired.removed_guards);
    assert_eq!(refs(&c.f), before_refs);
    release_lock(&acquired).unwrap();
    assert_eq!(fs::read_dir(c.path.parent().unwrap()).unwrap().count(), 0);
}

#[test]
fn racing_reapers_of_the_same_dead_guard_return_exactly_one_removal_and_one_winner() {
    let c = guard_case();
    // The decorator still asks the kernel. Its barrier makes both takers observe this same dead guard
    // before either can remove it; there is no sleep, polling or elapsed-time assertion.
    let probe = SynchronizedProbe { pid: c.dead_guard.pid, arrivals: AtomicUsize::new(0), barrier: Barrier::new(2) };
    let one = holder("taker-one");
    let two = holder("taker-two");
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(|| acquire_lock(&c.f.common, &LockName::Land, &one, &probe));
        let second = scope.spawn(|| acquire_lock(&c.f.common, &LockName::Land, &two, &probe));
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert!(probe.arrivals.load(Ordering::SeqCst) >= 2);
    let mut winners = Vec::new();
    let mut removals = Vec::new();
    let mut refused = 0;
    for result in results {
        match result {
            Ok(lock) => {
                removals.extend(lock.removed_guards.clone());
                winners.push(lock);
            }
            Err(error) => {
                removals.extend_from_slice(error.removed_guards());
                assert!(matches!(error, LockError::Held { .. } | LockError::TakeoverInProgress { .. } | LockError::Contended { .. }), "{error:?}");
                refused += 1;
            }
        }
    }
    assert_eq!(winners.len(), 1);
    assert_eq!(refused, 1);
    assert_eq!(removals, vec![c.dead_guard.clone()], "the removed guard must be reported once, even if its remover lost");
    // The retry may acquire while the stale primary lock is absent; that is a fresh acquire, not a takeover.
    if let Some(previous) = &winners[0].took_over {
        assert_eq!(previous, &c.stale);
    }
    assert_eq!(decode_lock_record(&fs::read_to_string(&c.path).unwrap()).unwrap(), winners[0].record);
    assert!(!c.guard.exists());
    assert!(!reap_path(&c).exists());
    assert_eq!(fs::read_dir(c.path.parent().unwrap()).unwrap().count(), 1);
    log_removals(&c, &removals);
    release_lock(&winners[0]).unwrap();
}

#[test]
fn crash_left_reap_guard_refuses_even_when_its_holder_is_dead_here() {
    let c = guard_case();
    let reap = reap_path(&c);
    let crashed = LockRecord { key: "crashed-reaper".into(), ..c.dead_guard.clone() };
    write_record(&reap, &crashed);
    let before_lock = fs::read(&c.path).unwrap();
    let before_guard = fs::read(&c.guard).unwrap();
    let before_reap = fs::read(&reap).unwrap();
    let error = acquire_lock(&c.f.common, &LockName::Land, &c.me, &KillPidProbe).unwrap_err();
    assert!(error.removed_guards().is_empty());
    match error {
        LockError::TakeoverInProgress { path, holder, .. } => {
            assert_eq!(path, reap);
            assert_eq!(holder, crashed);
        }
        other => panic!("expected crash-left reap refusal: {other:?}"),
    }
    assert_eq!(fs::read(&c.path).unwrap(), before_lock);
    assert_eq!(fs::read(&c.guard).unwrap(), before_guard);
    assert_eq!(fs::read(&reap).unwrap(), before_reap);
    assert!(!oplog_path(&c.f.common).exists());
}

#[test]
fn a_removed_guard_is_returned_and_can_be_logged_even_when_acquire_then_refuses() {
    let c = guard_case();
    let replacement = holder("competing-live-owner");
    // A competing writer replaces the main lock after acquire read it, at the guard PID observation.
    // Only the file event is injected: every PID answer still comes from the real kernel probe.
    let probe = ReplacingProbe {
        pid: c.dead_guard.pid, path: c.path.clone(), replacement: replacement.clone(), replaced: AtomicBool::new(false),
    };
    let error = acquire_lock(&c.f.common, &LockName::Land, &c.me, &probe).unwrap_err();
    assert!(probe.replaced.load(Ordering::SeqCst));
    assert_eq!(error.removed_guards(), std::slice::from_ref(&c.dead_guard));
    match &error {
        LockError::Held { path, holder, .. } => {
            assert_eq!(path, &c.path);
            assert_eq!(holder, &replacement);
        }
        other => panic!("expected refusal after guard removal: {other:?}"),
    }
    assert!(!c.guard.exists());
    assert!(!reap_path(&c).exists());
    assert_eq!(decode_lock_record(&fs::read_to_string(&c.path).unwrap()).unwrap(), replacement);
    log_removals(&c, error.removed_guards());
    assert_eq!(decode_lock_record(&fs::read_to_string(&c.path).unwrap()).unwrap(), replacement);
}

struct GuardCase {
    f: Fixture,
    path: PathBuf,
    guard: PathBuf,
    stale: LockRecord,
    dead_guard: LockRecord,
    me: LockRecord,
}

fn guard_case() -> GuardCase {
    let f = Fixture::new();
    // Keep both child handles until both are created, so the two recorded PIDs cannot be the same.
    let mut first = Command::new("true").spawn().unwrap();
    let mut second = Command::new("true").spawn().unwrap();
    let stale_pid = first.id();
    let guard_pid = second.id();
    assert_ne!(stale_pid, guard_pid);
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    assert!(!KillPidProbe.pid_alive(stale_pid));
    assert!(!KillPidProbe.pid_alive(guard_pid));
    let stale = LockRecord { pid: stale_pid, ..holder("stale-lock") };
    let dead_guard = LockRecord { pid: guard_pid, ..holder("dead-guard") };
    let me = holder("new-owner");
    let path = f.common.join("mortar/locks/land.lock");
    let guard = f.common.join("mortar/locks/land.lock.takeover");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    write_record(&path, &stale);
    write_record(&guard, &dead_guard);
    GuardCase { f, path, guard, stale, dead_guard, me }
}

fn reap_path(c: &GuardCase) -> PathBuf {
    // Literal SPEC-oplog/S-30 spelling, not the production path helper.
    c.f.common.join(format!("mortar/locks/land.lock.takeover.reap-{}-100", c.dead_guard.pid))
}

fn write_record(path: &std::path::Path, record: &LockRecord) {
    fs::write(path, encode_lock_record(record).unwrap()).unwrap();
}

fn log_removals(c: &GuardCase, removed: &[LockRecord]) {
    assert!(!oplog_path(&c.f.common).exists(), "acquire returns evidence; it must not log on its own");
    assert_eq!(removed, std::slice::from_ref(&c.dead_guard));
    let before_refs = refs(&c.f);
    let mut sequence = 0;
    let mut next_id = || { sequence += 1; format!("guard-log-{sequence}") };
    let ops = guard_removal_op(&c.path, &c.me, removed, &mut next_id, 200);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].op_id, "guard-log-1");
    assert_eq!(ops[0].kind, "takeover-guard-removal");
    assert_eq!(ops[0].actor, format!("{}@{}", c.me.pid, c.me.host));
    assert!(ops[0].moves.is_empty());
    for detail in [c.guard.display().to_string(), c.dead_guard.pid.to_string(), c.dead_guard.host.clone(), c.dead_guard.key.clone()] {
        assert!(ops[0].reason.contains(&detail), "missing {detail}: {}", ops[0].reason);
    }
    assert!(matches!(apply(&c.f.git, &c.f.common, &ops[0]).unwrap(), ApplyOutcome::Done(_)));
    let log = read_op_log(&c.f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.records[0].phase, OpPhase::Intent);
    assert_eq!(log.records[1].phase, OpPhase::Done);
    assert_eq!(log.records[1].op_id, "guard-log-1");
    assert_eq!(log.records[1].kind, "takeover-guard-removal");
    assert_eq!(log.records[1].reason, ops[0].reason);
    assert!(log.records[1].moves.is_empty());
    let bytes = fs::read(oplog_path(&c.f.common)).unwrap();
    let repeated = guard_removal_op(&c.path, &c.me, removed, &mut next_id, 201);
    assert_eq!(repeated[0].op_id, "guard-log-2");
    assert_eq!(apply(&c.f.git, &c.f.common, &repeated[0]).unwrap(), ApplyOutcome::Repeated(log.records[1].clone()));
    assert_eq!(fs::read(oplog_path(&c.f.common)).unwrap(), bytes);
    assert_eq!(refs(&c.f), before_refs);
}

struct SynchronizedProbe {
    pid: u32,
    arrivals: AtomicUsize,
    barrier: Barrier,
}

impl PidProbe for SynchronizedProbe {
    fn pid_alive(&self, pid: u32) -> bool {
        if pid == self.pid && self.arrivals.fetch_add(1, Ordering::SeqCst) < 2 {
            self.barrier.wait();
        }
        KillPidProbe.pid_alive(pid)
    }
}

struct ReplacingProbe {
    pid: u32,
    path: PathBuf,
    replacement: LockRecord,
    replaced: AtomicBool,
}

impl PidProbe for ReplacingProbe {
    fn pid_alive(&self, pid: u32) -> bool {
        if pid == self.pid && !self.replaced.swap(true, Ordering::SeqCst) {
            write_record(&self.path, &self.replacement);
        }
        KillPidProbe.pid_alive(pid)
    }
}
