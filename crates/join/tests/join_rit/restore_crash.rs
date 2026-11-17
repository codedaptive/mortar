//! WO-join-rit-9 / S-142(1): real resolve ref transaction -> SIGKILL/reap -> Oplog recovery/undo -> Join restore.
//! Uses the same after-transaction boundary as resolution/crashes.rs: recovery closes its durable open intent.

use super::restore_support::{caller, Scenario, ACTOR, REQUEST};
use gitio::{Git, Oid};
use join::JoinRunError;
use oplog::{KillPidProbe, OpPhase, PidProbe};
use std::{fs, os::unix::process::ExitStatusExt, path::PathBuf};
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

#[test]
fn restore_after_killed_resolve_and_real_undo_overwrites_only_that_resolves_finished_state() {
    let r = Scenario::paused(); let paused = r.state_bytes();
    let shim = r.common().join("restore-kill-git.sh");
    write_git_kill_shim(&shim).unwrap();
    let pid_file = r.common().join("restore-worker.pid");
    let mut command = test_child("restore_crash::resolve_worker").unwrap();
    command.envs(r.f.git.env.iter().cloned())
        .env("RESTORE_RIT_REPO", &r.f.repo.path).env("RESTORE_RIT_PROGRAM", &shim)
        .env("RESTORE_RIT_TREE", r.caller_tree.as_str()).env("RESTORE_RIT_PID", &pid_file)
        .env("MORTAR_TEST_CRASH", "after");
    let output = run_with_kill_gate(&mut command, &r.common()).unwrap();
    assert_eq!(output.status.signal(), Some(9), "{output:?}");
    let pid: u32 = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    assert!(!KillPidProbe.pid_alive(pid), "actual worker reaped before releasing its git wrapper");
    let leftover = r.state_bytes();
    assert_ne!(leftover, paused, "kill must leave the resolve's saved final fields, not the original pause");
    let to = r.f.git.read_ref(REQUEST).unwrap().unwrap(); assert_ne!(to, r.tip);
    let text = std::str::from_utf8(&leftover).unwrap();
    assert!(text.contains(&format!("\"rebased_to\":\"{to}\"")));
    assert!(text.contains(&format!("\"folder_tree\":\"{}\"", r.folder_tree)));
    let log = oplog::read_valued_log(&r.common()).unwrap();
    assert_eq!(log.records.len(), 1, "the killed transaction has an intent but no done record yet");
    assert_eq!(log.records[0].record.phase, OpPhase::Intent);
    // Caller lock/recover closes that real transaction before either undo or restoration.
    r.call(|| Ok(())).unwrap();
    let op = r.done(); r.assert_values(&op, &paused);
    r.undo(&op); assert_eq!(r.state_bytes(), leftover, "oplog undo moves refs, not Join's state file");
    let before = r.observe();
    assert_eq!(r.restore(&op).unwrap(), r.restored(false));
    assert_eq!(r.state_bytes(), paused, "overwrite only the undone resolve's own saved final state");
    assert_eq!(r.observe(), before);
    assert_eq!(r.assert_rebased(r.resolve().unwrap()), to);
    let records = oplog::read_op_log(&r.common()).unwrap().records;
    // Recovering the dead worker's caller lock also logs exactly one takeover, before closing the resolve.
    let phases: Vec<_> = records.iter().map(|v| (v.kind.as_str(), v.phase.clone())).collect();
    assert_eq!(phases, vec![
        ("join-rebase", OpPhase::Intent), ("lock-takeover", OpPhase::Intent), ("lock-takeover", OpPhase::Done),
        ("join-rebase", OpPhase::Done), ("undo", OpPhase::Intent), ("undo", OpPhase::Done),
        ("join-rebase", OpPhase::Intent), ("join-rebase", OpPhase::Done),
    ]);
}

#[test]
#[ignore = "actual SIGKILL resolve worker, invoked and reaped by the restore crash parent"]
fn resolve_worker() {
    let path = |name| PathBuf::from(std::env::var_os(name).unwrap());
    let repo = path("RESTORE_RIT_REPO");
    fs::write(path("RESTORE_RIT_PID"), std::process::id().to_string()).unwrap();
    let mut git = Git::new(&repo); git.program = path("RESTORE_RIT_PROGRAM");
    for name in ["GIT_CONFIG_GLOBAL", "GIT_CONFIG_NOSYSTEM", "GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_AUTHOR_DATE",
        "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL", "GIT_COMMITTER_DATE", "MORTAR_TEST_CRASH"] {
        git.env.push((name.into(), std::env::var_os(name).unwrap()));
    }
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let tree = Oid::parse(&std::env::var("RESTORE_RIT_TREE").unwrap()).unwrap();
    let result: Result<_, JoinRunError> = caller(&git, &repo, || join::resolve(&git, &repo, &tree, ACTOR));
    panic!("resolve worker must be killed at the real ref transaction, not return {result:?}");
}
