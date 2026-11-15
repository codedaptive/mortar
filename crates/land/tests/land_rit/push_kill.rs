//! Public publish -> real push boundary -> SIGKILL; Testkit waits/reaps the worker before releasing its git child.
//! No seeded intent or private producer/codec: the worker itself opens the actual remote-marked intent.
use crate::{owed_ops::refs, push_fixture::{Prepared, ACTOR}};
use gitio::Git;
use oplog::{KillPidProbe, LockName};
use std::{fs, os::unix::process::ExitStatusExt, path::PathBuf};

const SHIM: &str = r#"#!/bin/sh
set -eu
if [ "${3-}" = push ]; then
  if [ "${RIT_KILL_MODE-}" != before ]; then
    printf 'push\n' >> "$RIT_PUSH_TRACE"
    "$RIT_REAL_GIT" "$@"
    if [ "${RIT_KILL_MODE-}" != after ]; then exit 0; fi
  fi
  worker=$PPID
  printf '%s' "$worker" > "$RIT_KILLED_PID"
  printf 'armed\n' > "$TESTKIT_GATE_ARMED"
  kill -KILL "$worker"
  IFS= read -r release < "$TESTKIT_GATE_RELEASE"
  [ "$release" = release ] || exit 97
  if kill -0 "$worker" 2>/dev/null; then
    printf 'not-reaped\n' > "$TESTKIT_GATE_ACK"
  else
    printf 'reaped\n' > "$TESTKIT_GATE_ACK"
  fi
  exit 0
fi
exec "$RIT_REAL_GIT" "$@"
"#;

pub fn interrupt(p: &Prepared, mode: &str) -> Git {
    assert!(mode == "before" || mode == "after");
    let git = p.traced_git(); testkit::write_git_shim(&PathBuf::from(&git.program), SHIM.as_bytes()).unwrap();
    let pid = p.f.git.repo.join("rit3-killed-pid");
    let mut child = testkit::test_child("push_kill::rit_remote_push_worker").unwrap();
    child.env_clear().env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", &p.f.git.repo).env("TMPDIR", &p.f.git.repo)
        .envs(git.env.iter().cloned()).env("RIT_REPO", &p.f.git.repo)
        .env("RIT_GIT_PROGRAM", &git.program).env("RIT_KILL_MODE", mode).env("RIT_KILLED_PID", &pid);
    let output = testkit::run_with_kill_gate(&mut child, &p.f.git.repo).unwrap();
    assert_eq!(output.status.signal(), Some(9), "actual publish worker must be killed: {output:?}");
    assert!(String::from_utf8(fs::read(pid).unwrap()).unwrap().parse::<u32>().unwrap() > 0);
    // Success from run_with_kill_gate includes the wrapper's explicit kernel-reaped acknowledgement.
    assert_eq!(p.trace_bytes(), if mode == "before" { b"".as_slice() } else { b"push\n".as_slice() });
    git
}

#[test]
#[ignore = "subprocess entry only; parents select it with Testkit and verify its SIGKILL/reap"]
fn rit_remote_push_worker() {
    let mut git = Git::new(std::env::var_os("RIT_REPO").unwrap());
    git.program = std::env::var_os("RIT_GIT_PROGRAM").unwrap().into();
    let common = git.common_dir().unwrap();
    oplog::with_lock(&git, &common, &LockName::Land, "rit3-killed-worker", &KillPidProbe, || {
        oplog::recover(&git, &common).unwrap();
        land::publish(&git, &refs(), ACTOR)
    }).unwrap();
    panic!("the selected push boundary must kill this worker before publish returns");
}

/// Recovery is deliberately given a Git that refuses EVERY process: marked remote intents need no Git reads.
/// This is an outcome assertion on a real durable intent, not a fake publish result.
pub fn recover_leaves_open(p: &Prepared) {
    let path = p.f.common.join("mortar/oplog.jsonl");
    let no_git = p.f.git.repo.join("rit3-no-recovery-git");
    let called = p.f.git.repo.join("rit3-unexpected-recovery-call");
    testkit::write_git_shim(&no_git, b"#!/bin/sh\nprintf 'called\\n' >> \"$RIT_RECOVERY_CALLED\"\nexit 97\n").unwrap();
    let mut git = p.f.git.clone(); git.program = no_git;
    git.env.push(("RIT_RECOVERY_CALLED".into(), called.clone().into_os_string()));
    crate::push_fixture::with_lock(p, || {
        // Acquiring the dead worker's lock legitimately logs lock-takeover BEFORE this closure.
        // The invariance belongs around recover itself, after that required takeover has finished.
        let before = fs::read(&path).unwrap();
        assert_eq!(oplog::recover(&git, &p.f.common).unwrap(), vec![]);
        assert_eq!(oplog::recover(&git, &p.f.common).unwrap(), vec![]);
        assert_eq!(fs::read(&path).unwrap(), before, "local recovery must not close/rewrite the marked push");
        Ok::<_, oplog::OplogError>(())
    });
    assert!(!called.exists(), "local recovery must not even invoke Git for this marked remote intent");
    let plain = oplog::read_op_log(&p.f.common).unwrap().records;
    assert_eq!(oplog::open_intents(&plain), vec![&p.pushes()[0].record]);
    let marked = oplog::read_marked_log(&p.f.common).unwrap().records;
    let valued = oplog::read_valued_log(&p.f.common).unwrap().records;
    assert_eq!(plain, marked.iter().map(|m| m.record.clone()).collect::<Vec<_>>());
    assert_eq!(valued.iter().map(|v| v.record.clone()).collect::<Vec<_>>(), plain);
    assert!(marked.iter().zip(&valued).all(|(m, v)| m.values == v.values));
}
