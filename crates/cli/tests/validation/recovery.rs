use crate::support::*;
use oplog::{KillPidProbe, LockName, LockRecord, Op, OpPhase, RefMove};
use std::fs;

#[test]
fn rit_live_join_lock_refuses_join_and_save_with_exact_holder_and_no_act() {
    for verb in ["join", "save"] {
        let f = Fixture::new(); f.write("src/file", b"would be saved if act ran\n");
        let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "rit-live-holder".into(), since: 123 };
        let held = oplog::acquire_lock(&f.common, &LockName::Join("core".into()), &holder, &KillPidProbe).unwrap();
        let lock_bytes = fs::read(f.lock_path()).unwrap(); let before = f.observe();
        let args = if verb == "save" { vec!["--json", "save", "Must not run"] } else { vec!["--json", "join"] };
        let busy = f.report(&args, verb, 3, "busy");
        assert_eq!(busy.number("holder_pid"), u64::from(holder.pid)); assert_eq!(busy.text("holder_host"), holder.host);
        assert_eq!(busy.text("holder_key"), "rit-live-holder"); assert_eq!(busy.number("since"), 123);
        assert!(busy.lines().contains("nothing ran")); assert_eq!(f.observe(), before);
        assert_eq!(fs::read(f.lock_path()).unwrap(), lock_bytes); assert_eq!(f.git.read_ref(SNAP).unwrap(), None);
        oplog::release_lock(&held).unwrap(); assert!(!f.lock_path().exists());
    }
}

#[test]
fn rit_open_intent_is_recovered_before_snapshot_act_and_the_lock_is_released() {
    for moved in [false, true] {
        let f = Fixture::new(); let start = f.records().len();
        let op = Op { op_id: "rit-open-intent".into(), key: "rit-open-key".into(), kind: "rit-recovery".into(),
            actor: "recovery-fixture".into(), reason: "prove recover precedes the verb".into(), at: 100,
            moves: vec![RefMove { refname: "refs/checkpoints/recovery".into(), old: None, new: f.base.clone() }] };
        oplog::append_op_record(&f.common, &oplog::intent_record(&op)).unwrap();
        if moved { raw(&f.git, &["update-ref", "refs/checkpoints/recovery", f.base.as_str()]); }
        let report = f.report(&["--json", "join"], "join", 0, "joined"); assert!(!report.flag("saved_written"));
        let records = f.records();
        assert_eq!(records[start].phase, OpPhase::Intent);
        assert_eq!(records[start + 1].op_id, "rit-open-intent");
        assert_eq!(records[start + 1].phase, if moved { OpPhase::Done } else { OpPhase::Failed });
        assert_eq!(records[start + 2].kind, "snapshot", "recover closes BEFORE any act writes its snapshot");
        assert_eq!(oplog::open_intent(&f.common, "rit-open-intent").unwrap(), None);
        assert_eq!(f.git.read_ref("refs/checkpoints/recovery").unwrap(), if moved { Some(f.base.clone()) } else { None });
        assert!(!f.lock_path().exists());
    }
}

#[test]
fn rit_unreadable_oplog_is_exit_five_naming_path_without_act_and_releases_lock() {
    let f = Fixture::new(); let path = oplog::oplog_path(&f.common);
    fs::remove_file(&path).unwrap(); fs::create_dir(&path).unwrap();
    let report = f.report(&["--json", "join"], "join", 5, "failed");
    assert!(report.lines().contains(path.to_str().unwrap())); assert_eq!(f.head(), f.base);
    assert_eq!(f.git.read_ref(SNAP).unwrap(), None); assert!(path.is_dir()); assert!(!f.lock_path().exists());
    // In the binary flow recover also reads this log before read_pending_move; this checks the observable failure,
    // not an isolated IO arm of the detector that could only fail after a separate file-system race.
}

#[test]
fn rit_absent_oplog_means_no_pending_move_and_join_can_initialize_the_snapshot() {
    let f = Fixture::new(); fs::remove_file(oplog::oplog_path(&f.common)).unwrap();
    let report = f.report(&["--json", "join"], "join", 0, "joined");
    assert!(report.is_null("finished_move_from")); assert!(report.is_null("finished_move_to"));
    assert_eq!(f.head(), f.base); assert_eq!(f.records().len(), 2);
    assert!(f.records().iter().all(|r| r.kind == "snapshot")); assert!(!f.lock_path().exists());
}

#[test]
fn rit_later_save_or_foreign_request_tip_is_not_mistaken_for_the_logged_rebase_move() {
    for kind in ["later-save", "foreign-ref"] {
        let f = Fixture::new(); let (old, _, _) = f.pending_rebase();
        let expected = if kind == "later-save" {
            f.report(&["--json", "join"], "join", 0, "joined");
            f.write("src/file", b"later explicit save\n"); f.report(&["--json", "save", "Save later"], "save", 0, "saved");
            "joined"
        } else {
            raw(&f.git, &["update-ref", REQUEST, old.as_str()]); "rebased"
        };
        let report = f.report(&["--json", "join"], "join", 0, expected);
        assert!(report.is_null("finished_move_from"), "{kind}"); assert!(report.is_null("finished_move_to"), "{kind}");
        assert!(!f.lock_path().exists());
    }
}
