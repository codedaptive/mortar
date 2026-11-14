use super::*;
use oplog::PidProbe;
use std::os::unix::process::ExitStatusExt;
use testkit::{run_with_kill_gate, test_child, write_git_shim};

#[test]
fn killed_after_rebased_to_is_saved_before_the_op_repeats_without_replaying_or_moving_twice() {
    kill_and_repeat("before");
}

#[test]
fn killed_after_the_ref_transaction_recovers_then_finishes_state_cleanup_without_another_move() {
    kill_and_repeat("after");
}

fn kill_and_repeat(mode: &str) {
    let r = Paused::new("none"); r.first();
    fs::write(r.f.repo.path.join(&r.path), b"resolved\n").unwrap();
    let tree = r.caller_tree();
    let expected_tree = r.f.tree(&[(&r.path, b"resolved\n"), ("outside", b"advanced zone work\n"), ("second", b"base second\n")]);
    // Independent raw-Git commit oracle from the specified tree/parent/message/author, not resolve's output.
    let expected = r.f.commit(&expected_tree, &[r.zone.clone()], "conflicted message\n", &author("Zoë", "1700000100 -0530"));
    kill_at_boundary(&r, &tree, mode);
    assert!(r.state_text().contains(&format!("\"rebased_to\":\"{expected}\"")));
    assert_eq!(r.f.git.read_ref(REQUEST_REF).unwrap(), Some(if mode == "before" { r.tip.clone() } else { expected.clone() }));
    // Saved progress must finish even if markers reappear and a caller tree is unusable: no second replay/read.
    fs::write(r.f.repo.path.join(&r.path), b"<<<<<<< unrelated later local edit\n").unwrap();
    let invalid_tree = Oid::parse(&"f".repeat(40)).unwrap();
    assert_eq!(rebased(r.resolve(&invalid_tree).unwrap(), &r.tip), expected);
    assert_one_rebase(&r, &expected);
    assert_eq!(fs::read(r.f.repo.path.join(&r.path)).unwrap(), b"<<<<<<< unrelated later local edit\n");
    let log = r.log();
    assert!(matches!(r.resolve(&invalid_tree), Err(JoinRunError::NothingToResolve { .. })));
    assert_eq!(r.log(), log);
}

/// Shared actual kill/reap protocol for both the original single-round tests and S-87's two-round handoff.
pub(super) fn kill_at_boundary(r: &Paused, tree: &Oid, mode: &str) {
    let common = r.f.repo.path.join(".git");
    let shim = common.join("kill-resolve-boundary.sh"); let marker = common.join("resolve-boundary"); let pid_file = common.join("resolve-worker.pid");
    write_git_shim(&shim, br#"#!/bin/sh
set -eu
armed=0
if [ "$MORTAR_TEST_RESOLVE_MODE" = before ]; then
  case " $* " in
    *' --git-common-dir '*)
      if grep -q '"rebased_to":"[0-9a-f]' "$MORTAR_TEST_RESOLVE_STATE"; then armed=1; fi;;
  esac
elif [ "${3-}" = update-ref ] && [ "${4-}" = --stdin ]; then
  git "$@"
  armed=1
fi
if [ "$armed" = 1 ]; then
  [ "$PPID" = "$MORTAR_TEST_WORKER_PID" ] || exit 98
  printf '%s' "$MORTAR_TEST_RESOLVE_MODE" >"$MORTAR_TEST_RESOLVE_MARKER"
  printf 'armed\n' >"$TESTKIT_GATE_ARMED"
  kill -KILL "$MORTAR_TEST_WORKER_PID"
  IFS= read -r release <"$TESTKIT_GATE_RELEASE"
  [ "$release" = release ] || exit 97
  if kill -0 "$MORTAR_TEST_WORKER_PID" 2>/dev/null; then
    printf 'not-reaped\n' >"$TESTKIT_GATE_ACK"
  else
    printf 'reaped\n' >"$TESTKIT_GATE_ACK"
  fi
  exit 0
fi
exec git "$@"
"#).unwrap();
    let mut command = test_child("resolution::crashes::resolve_crash_worker").unwrap();
    command.envs(r.f.git.env.iter().cloned());
    command.env("MORTAR_TEST_RESOLVE_REPO", &r.f.repo.path).env("MORTAR_TEST_RESOLVE_PROGRAM", &shim)
        .env("MORTAR_TEST_RESOLVE_TREE", tree.as_str()).env("MORTAR_TEST_RESOLVE_MODE", mode)
        .env("MORTAR_TEST_RESOLVE_STATE", r.state_path()).env("MORTAR_TEST_RESOLVE_MARKER", &marker)
        .env("MORTAR_TEST_RESOLVE_PID", &pid_file);
    let output = run_with_kill_gate(&mut command, &common).unwrap();
    assert_eq!(output.status.signal(), Some(9), "{output:?}");
    assert_eq!(fs::read(&marker).unwrap(), mode.as_bytes());
    let pid: u32 = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    assert!(!KillPidProbe.pid_alive(pid), "actual worker must be reaped, not merely signalled");
}

#[test]
#[ignore = "actual SIGKILL resolve worker, invoked and reaped by the crash parents"]
fn resolve_crash_worker() {
    let path = |name| PathBuf::from(std::env::var_os(name).unwrap());
    let repo = path("MORTAR_TEST_RESOLVE_REPO");
    fs::write(path("MORTAR_TEST_RESOLVE_PID"), std::process::id().to_string()).unwrap();
    let mut git = Git::new(&repo); git.program = path("MORTAR_TEST_RESOLVE_PROGRAM");
    git.env.extend([
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ("GIT_COMMITTER_NAME".into(), "Replay Committer".into()), ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ("GIT_COMMITTER_DATE".into(), "1700000999 +0000".into()),
        ("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()),
    ]);
    let common = repo.join(".git");
    let tree = Oid::parse(&std::env::var("MORTAR_TEST_RESOLVE_TREE").unwrap()).unwrap();
    with_lock(&git, &common, &LockName::Join(JOIN_ZONE.into()), ACTOR, &KillPidProbe, || {
        oplog::recover(&git, &common).map_err(JoinRunError::Oplog)?;
        join::resolve(&git, &repo, &tree, ACTOR)
    }).unwrap();
    panic!("resolve worker must be killed at the chosen real boundary");
}
