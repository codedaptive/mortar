//! S-85/S-143: CLI status -> real Oplog recovery decisions and Gitio, without settling or moving anything.
//! Owned Fixture repositories; Testkit kills/reaps the actual CLI after its real rebase ref transaction.
use crate::support::{raw, script, shim_path, Fixture, REQUEST, ZONE};
use oplog::{KillPidProbe, OpPhase, OpRecord, PidProbe, RefMove};
use std::{fs, os::unix::process::ExitStatusExt, process::{Command, Stdio}};
use testkit::{run_with_kill_gate, write_git_kill_shim};

#[test]
fn status_reads_a_killed_rebases_open_intent_as_stalled_without_writing_then_join_finishes_once() {
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(false);
    let shim = f.common.join("precheck-status-kill.sh"); write_git_kill_shim(&shim).unwrap();
    let bin = f.common.join("precheck-status-bin"); fs::create_dir(&bin).unwrap();
    // Reset PATH only inside the Git shim so Testkit's own `git` dispatch cannot recurse into this adapter.
    script(&bin.join("git"), b"#!/bin/sh\nPATH=\"$PSR_REAL_PATH\" exec \"$PSR_SHIM\" \"$@\"\n");
    let template = f.command(&f.request, &["--json", "join"]);
    let mut command = Command::new("/bin/sh");
    command.current_dir(&f.request).env_clear()
        .envs(template.get_envs().filter_map(|(key, value)| value.map(|value| (key, value))))
        // exec preserves this pid as the actual Cargo-built CLI that holds the caller lock.
        .args(["-c", "export MORTAR_TEST_WORKER_PID=$$; exec \"$@\"", "status-rebase-worker",
            env!("CARGO_BIN_EXE_mortar"), "--json", "join"])
        .env("PATH", shim_path(&bin)).env("PSR_REAL_PATH", std::env::var_os("PATH").unwrap())
        .env("PSR_SHIM", &shim).env("MORTAR_TEST_CRASH", "after").stdin(Stdio::null());
    let killed = run_with_kill_gate(&mut command, &f.common).unwrap();
    assert_eq!(killed.status.signal(), Some(9), "{killed:?}");
    let holder_bytes = fs::read(f.lock_path()).unwrap();
    let holder = oplog::decode_lock_record(std::str::from_utf8(&holder_bytes).unwrap()).unwrap();
    assert!(!KillPidProbe.pid_alive(holder.pid));
    let new = f.head(); assert_ne!(new, old);
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(zone.clone()));
    assert_eq!(f.git.commit_info(&new).unwrap().parents, vec![zone]);
    let records = f.records();
    let intents: Vec<_> = records.iter().filter(|r| r.kind == "join-rebase").collect();
    assert_eq!(intents.len(), 1); let intent = intents[0]; assert_eq!(intent.phase, OpPhase::Intent);
    assert_eq!(intent.moves, vec![RefMove { refname: REQUEST.into(), old: Some(old.clone()), new: new.clone() }]);
    assert_eq!(oplog::open_intent(&f.common, &intent.op_id).unwrap(), Some(intent.moves.clone()));
    assert!(!f.request.join("src/upstream").exists());
    let index = f.git.git_dir(&f.request).unwrap().join("index");
    let before_index = fs::read(&index).unwrap(); let before = f.observe();
    let report = f.report(&["--json", "status"], "status", 0, "stalled");
    assert_eq!(report.text("stalled_from"), old.as_str()); assert_eq!(report.text("stalled_to"), new.as_str());
    assert_eq!(report.text("next"), "mr join");
    assert_eq!(f.observe(), before); assert_eq!(fs::read(&index).unwrap(), before_index);
    assert_eq!(fs::read(f.lock_path()).unwrap(), holder_bytes, "read-only status must not take over the dead lock");
    assert_eq!(oplog::open_intent(&f.common, &intent.op_id).unwrap(), Some(intent.moves.clone()));
    let finished = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(finished.text("finished_move_from"), old.as_str());
    assert_eq!(finished.text("finished_move_to"), new.as_str());
    assert_eq!(f.head(), new); assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    assert_eq!(f.records().iter().filter(|r| r.kind == "join-rebase" && r.phase == OpPhase::Done).count(), 1);
    assert_eq!(oplog::open_intent(&f.common, &intent.op_id).unwrap(), None); assert!(!f.lock_path().exists());
    let before = f.observe(); f.report(&["--json", "join"], "join", 0, "joined"); assert_eq!(f.observe(), before);
}

#[test]
fn status_leaves_a_true_mixed_recovery_intent_open_without_inventing_a_stalled_move() {
    let f = Fixture::new(); let old = f.save_topic();
    let new = f.commit(&old, &[("src/upstream", b"mixed new tree\n")], "partially moved refs\n");
    const OTHER: &str = "refs/heads/mixed-other";
    raw(&f.git, &["update-ref", REQUEST, new.as_str()]);
    raw(&f.git, &["update-ref", OTHER, old.as_str()]);
    let intent = OpRecord {
        op_id: "precheck-status-mixed".into(), key: "join-rebase:precheck-status-mixed".into(),
        kind: "join-rebase".into(), actor: "core/task".into(), reason: "mixed recovery fixture".into(),
        at: 1_700_000_000, phase: OpPhase::Intent, result: None,
        moves: vec![
            RefMove { refname: REQUEST.into(), old: Some(old.clone()), new: new.clone() },
            RefMove { refname: OTHER.into(), old: Some(old.clone()), new: new.clone() },
        ],
    };
    oplog::append_valued_record(&f.common, &intent, &[]).unwrap();
    assert_eq!(f.git.read_ref(REQUEST).unwrap(), Some(new)); assert_eq!(f.git.read_ref(OTHER).unwrap(), Some(old));
    let before = f.observe();
    let index = f.git.git_dir(&f.request).unwrap().join("index"); let before_index = fs::read(&index).unwrap();
    let report = f.report(&["--json", "status"], "status", 0, "unsaved");
    assert!(report.is_null("stalled_from")); assert!(report.is_null("stalled_to"));
    assert_eq!(report.text("next"), "mr save \"<sentence>\"");
    assert_eq!(f.observe(), before); assert_eq!(fs::read(&index).unwrap(), before_index);
    assert_eq!(oplog::open_intent(&f.common, &intent.op_id).unwrap(), Some(intent.moves.clone()));
    assert!(matches!(oplog::recover(&f.git, &f.common), Err(oplog::OplogError::Mixed { .. })));
    assert_eq!(f.observe(), before); assert!(!f.lock_path().exists());
}
