//! S-148: pre-check of a real resolve's finished state left by SIGKILL, then Oplog undo and Join restore.
//! Reuses the existing resolve worker and Testkit's after-transaction kernel-reap boundary, not a fabricated state.
use super::restore_support::{Scenario, REQUEST};
use oplog::{KillPidProbe, OpPhase, PidProbe};
use std::{fs, os::unix::process::ExitStatusExt};
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

#[test]
fn precheck_of_a_killed_resolves_own_finished_state_is_ok_then_undo_and_restore_succeed() {
    let r = Scenario::paused(); let paused = r.state_bytes();
    let shim = r.common().join("precheck-kill-git.sh"); write_git_kill_shim(&shim).unwrap();
    let pid_file = r.common().join("precheck-worker.pid");
    let mut command = test_child("restore_crash::resolve_worker").unwrap();
    command.envs(r.f.git.env.iter().cloned())
        .env("RESTORE_RIT_REPO", &r.f.repo.path).env("RESTORE_RIT_PROGRAM", &shim)
        .env("RESTORE_RIT_TREE", r.caller_tree.as_str()).env("RESTORE_RIT_PID", &pid_file)
        .env("MORTAR_TEST_CRASH", "after");
    let output = run_with_kill_gate(&mut command, &r.common()).unwrap();
    assert_eq!(output.status.signal(), Some(9), "{output:?}");
    let pid: u32 = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    assert!(!KillPidProbe.pid_alive(pid));
    let leftover = r.state_bytes(); let to = r.f.git.read_ref(REQUEST).unwrap().unwrap();
    assert_ne!(leftover, paused); assert_ne!(to, r.tip);
    let text = std::str::from_utf8(&leftover).unwrap();
    assert!(text.contains(&format!("\"rebased_to\":\"{to}\"")));
    assert!(text.contains(&format!("\"folder_tree\":\"{}\"", r.folder_tree)));
    let log = oplog::read_valued_log(&r.common()).unwrap();
    assert_eq!(log.records.len(), 1); assert_eq!(log.records[0].record.phase, OpPhase::Intent);
    // The caller recovers the actual ref transaction before pre-check, as it must before undo.
    r.call(|| Ok(())).unwrap();
    let op = r.done(); r.assert_values(&op, &paused); let before = r.observe();
    r.call(|| join::check_restore_join_state(&r.f.git, &r.f.repo.path, &op)).unwrap();
    assert_eq!(r.state_bytes(), leftover); assert_eq!(r.observe(), before);
    r.undo(&op); assert_eq!(r.state_bytes(), leftover);
    let before = r.observe();
    assert_eq!(r.restore(&op).unwrap(), r.restored(false));
    assert_eq!(r.state_bytes(), paused); assert_eq!(r.observe(), before);
    assert_eq!(r.assert_rebased(r.resolve().unwrap()), to);
}
