//! Actual killed-move and unmerged-index status boundaries (S-85, S-116 positive cases).
//! Orc#1651 explicitly permits the staged-after-finished case to wait until WO-cli-2g lands.
use crate::{observe::*, support::*};
use gitio::{IndexTreeError, RunOpts};
use oplog::{KillPidProbe, OpPhase, PidProbe};
use std::{fs, os::unix::process::ExitStatusExt};

#[test]
fn rit_status_reads_a_reaped_join_killed_after_done_without_recovering_or_moving_the_folder() {
    // actual mortar join -> Join/Oplog Done -> Testkit kill/reap gate -> mortar status -> Gitio copied index.
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(false);
    // Join may refresh stat-cache bytes; the killed movement must retain the old semantic index/tree.
    // Status's stricter RAW-index invariance is captured separately by status(), AFTER the kill.
    let old_index = raw(&f.local(), &["ls-files", "--stage", "-z"]);
    let old_tree = f.git.commit_info(&old).unwrap().tree;
    let dir = f.common.join("status-kill-git"); fs::create_dir(&dir).unwrap();
    let pid_file = f.common.join("status-killed-pid");
    // Initial snapshot's index reads are not the boundary: the real rebase op must exist first.
    // The shared gate waits/reaps the ACTUAL mortar child before releasing this wrapper; no polling or timer.
    testkit::write_git_shim(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${3-}" = read-tree ] && [ "${4-}" = --no-sparse-checkout ] &&
   grep -q '"kind":"join-rebase"' "$MORTAR_RIT_LOG" && mkdir "$MORTAR_RIT_ONCE" 2>/dev/null; then
  worker=$PPID
  printf '%s' "$worker" > "$MORTAR_RIT_PID"
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
exec "$MORTAR_RIT_REAL_GIT" "$@"
"#).unwrap();
    let mut command = f.command(&f.request, &["--json", "join"]);
    command.env("PATH", shim_path(&dir)).env("MORTAR_RIT_REAL_GIT", actual_git())
        .env("MORTAR_RIT_LOG", oplog::oplog_path(&f.common)).env("MORTAR_RIT_ONCE", f.common.join("status-kill-once"))
        .env("MORTAR_RIT_PID", &pid_file);
    let killed = testkit::run_with_kill_gate(&mut command, &f.common).unwrap();
    assert_eq!(killed.status.signal(), Some(9), "{killed:?}");
    let pid: u32 = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    assert!(!KillPidProbe.pid_alive(pid), "shared gate acknowledged kernel reaping before wrapper release");
    let holder = fs::read(f.lock_path()).unwrap();
    assert_eq!(oplog::decode_lock_record(std::str::from_utf8(&holder).unwrap()).unwrap().pid, pid);
    let new = f.head(); assert_ne!(new, old);
    assert_eq!(f.git.commit_info(&new).unwrap().parents, vec![zone]);
    assert_eq!(raw(&f.local(), &["ls-files", "--stage", "-z"]), old_index);
    assert_eq!(f.local().index_tree(&f.request).unwrap(), old_tree);
    assert!(!f.request.join("src/upstream").exists());
    let records = f.records(); let rebase: Vec<_> = records.iter().filter(|r| r.kind == "join-rebase").collect();
    assert_eq!(rebase.len(), 2); assert_eq!(rebase[0].phase, OpPhase::Intent); assert_eq!(rebase[1].phase, OpPhase::Done);
    assert_eq!(rebase[0].op_id, rebase[1].op_id);
    let report = status(&f, "stalled");
    assert_eq!(report.text("stalled_from"), old.as_str()); assert_eq!(report.text("stalled_to"), new.as_str());
    assert_eq!(report.text("next"), "mr join"); assert_eq!(fs::read(f.lock_path()).unwrap(), holder, "status ran no recover");
    f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(f.head(), new); assert!(!f.lock_path().exists());
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    let finished = status(&f, "joined");
    assert!(finished.is_null("stalled_from")); assert!(finished.is_null("stalled_to"));
    assert_eq!(finished.text("next"), "wait for the zone's tests");
    assert_eq!(f.records().iter().filter(|r| r.kind == "join-rebase" && r.phase == OpPhase::Done).count(), 1);
}

#[test]
fn rit_status_unmerged_real_index_is_conflicted_not_a_stalled_move() {
    // Status first exposes a real stall; actual Gitio index_tree must then classify copied unmerged entries.
    let f = Fixture::new(); let (old, new, _) = f.pending_rebase();
    let pending = status(&f, "stalled");
    assert_eq!(pending.text("stalled_from"), old.as_str()); assert_eq!(pending.text("stalled_to"), new.as_str());
    let mut ids = Vec::new();
    for bytes in [b"base\n".as_slice(), b"ours\n", b"theirs\n"] {
        ids.push(oid(&f.git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
            stdin: Some(bytes.to_vec()), ..RunOpts::default()
        }).unwrap().stdout));
    }
    let input = format!("0 {}\tsrc/file\n100644 {} 1\tsrc/file\n100644 {} 2\tsrc/file\n100644 {} 3\tsrc/file\n",
        "0".repeat(40), ids[0], ids[1], ids[2]);
    f.local().run(&["update-index", "--index-info"], &RunOpts { stdin: Some(input.into_bytes()), ..RunOpts::default() }).unwrap();
    assert!(matches!(f.local().index_tree(&f.request), Err(IndexTreeError::Unmerged { paths })
        if paths == vec![std::path::PathBuf::from("src/file")]));
    f.write("src/file", b"unmerged local work differs from the last snapshot\n");
    let report = status(&f, "unsaved");
    assert_eq!(report.strings("conflicted"), &["src/file"]);
    assert!(report.is_null("stalled_from")); assert!(report.is_null("stalled_to"));
    assert_eq!(report.text("next"), "mr save \"<sentence>\"");
}
