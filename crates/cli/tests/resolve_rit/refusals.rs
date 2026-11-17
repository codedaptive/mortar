use crate::{json::check, objects::assert_one_rebase, paused::Paused, support::*};
use oplog::{KillPidProbe, LockName, LockRecord};
use std::fs;

#[test]
fn rit_resolve_live_join_lock_is_exit_three_with_exact_holder_and_no_snapshot_or_state_changes() {
    let r = Paused::new(false); let f = &r.f; f.write("sub/keep", b"must not be snapshotted\n");
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "resolve-live-holder".into(), since: 321 };
    let held = oplog::acquire_lock(&f.common, &LockName::Join("core".into()), &holder, &KillPidProbe).unwrap();
    let bytes = fs::read(f.lock_path()).unwrap(); let before = f.observe(); let state = r.state_bytes();
    let report = f.report(&["--json", "resolve"], "resolve", 3, "busy");
    assert_eq!(report.number("holder_pid"), u64::from(holder.pid)); assert_eq!(report.text("holder_host"), holder.host);
    assert_eq!(report.text("holder_key"), "resolve-live-holder"); assert_eq!(report.number("since"), 321);
    assert!(report.lines().contains("nothing ran")); assert_eq!(f.observe(), before); assert_eq!(r.state_bytes(), state);
    assert_eq!(fs::read(f.lock_path()).unwrap(), bytes);
    assert_eq!(fs::read(f.request.join("sub/keep")).unwrap(), b"must not be snapshotted\n");
    oplog::release_lock(&held).unwrap(); r.assert_unmoved();
}

#[test]
fn rit_resolve_argument_is_exit_four_with_full_usage_and_no_lock_or_work() {
    let r = Paused::new(false); let f = &r.f; let before = f.observe(); let state = r.state_bytes();
    let report = f.report(&["--json", "resolve", "unexpected"], "resolve", 4, "usage");
    assert!(report.lines().contains("resolve takes no arguments"));
    assert_eq!(report.text("next"), "mr resolve [--key <operation key>] [--json]");
    assert_eq!(f.observe(), before); assert_eq!(r.state_bytes(), state); r.assert_unmoved();
}

#[test]
fn rit_real_attempt_folders_refuse_join_and_resolve_before_held_lock_recovery_or_snapshot() {
    let f = Fixture::new(); f.save_topic(); let attempts = f.root.join("attempts");
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_attempts(&setup(&f.git, &attempts), "core", "task", &["src".into(), "sub".into()], 2).unwrap();
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "must-not-inspect-lock".into(), since: 456 };
    let held = oplog::acquire_lock(&f.common, &LockName::Join("core".into()), &holder, &KillPidProbe).unwrap();
    let lock = fs::read(f.lock_path()).unwrap();
    for k in [1, 2] {
        let path = attempts.join(format!("try-{k}")); write(&path, "src/file", b"unsaved attempt work\n");
        let local = gitio::Git { repo: path.clone(), ..f.git.clone() }; let dir = local.git_dir(&path).unwrap();
        let record = fs::read(dir.join("mortar-folder.json")).unwrap();
        // An attempted resolve must reject the real Attempt(k) record without ever reading this poison state.
        let state = dir.join("mortar-join.json"); fs::write(&state, b"not a paused join\n").unwrap();
        let index = raw(&local, &["ls-files", "--stage", "-z"]); let before = f.observe();
        assert_eq!(folders::make::read_folder_record(&dir).unwrap().unwrap().kind, folders::FolderKind::Attempt(k));
        for verb in ["join", "resolve"] {
            let report = check(f.command(&path.join("src"), &["--json", verb]).output().unwrap(), verb, 1, "refused");
            assert_eq!(report.text("owner"), "this request");
            assert!(report.lines().contains(&format!("this is attempt folder {k} of request task: mr {verb} runs in the request's own folder")));
            assert!(report.lines().contains("nothing ran"));
            assert_eq!(report.text("next"), "keep or drop the attempts, then mr join in the request's folder");
            assert_eq!(f.observe(), before); assert_eq!(raw(&local, &["ls-files", "--stage", "-z"]), index);
            assert_eq!(fs::read(path.join("src/file")).unwrap(), b"unsaved attempt work\n");
            assert_eq!(fs::read(&state).unwrap(), b"not a paused join\n"); assert_eq!(fs::read(dir.join("mortar-folder.json")).unwrap(), record);
            assert_eq!(fs::read(f.lock_path()).unwrap(), lock);
            assert!(f.git.read_ref(&format!("refs/mortar/trysnap/core/task/{k}")).unwrap().is_none());
        }
    }
    oplog::release_lock(&held).unwrap(); assert!(!f.lock_path().exists());
}

#[test]
fn rit_resolve_move_refusal_names_zone_changed_path_and_snapshot_that_keeps_local_work() {
    let r = Paused::new(true); r.first(); let f = &r.f;
    f.write("src/file", b"chosen resolution\n"); f.write("sub/keep", b"local obstruction must survive\n");
    let state_before = fs::read_to_string(&r.state).unwrap();
    let state_before = state_before.strip_suffix('\n').expect("state file's trailing newline").to_string();
    let report = f.report(&["--json", "resolve"], "resolve", 1, "refused"); let new = f.head();
    assert_ne!(new, r.old); assert_eq!(report.text("owner"), "this request's folder");
    assert!(report.lines().contains("edits on paths the zone changed stand in the way"));
    assert!(report.lines().contains("  sub/keep"));
    let snap = f.git.read_ref(SNAP).unwrap().unwrap();
    assert!(report.lines().contains(&format!("they are kept in {SNAP} at {snap}")));
    assert_eq!(report.text("next"), "put those files back to the folder's version, then mr join");
    assert_eq!(f.blob(&snap, "sub/keep"), b"local obstruction must survive\n");
    assert_eq!(f.blob(&snap, "src/file"), b"chosen resolution\n");
    assert_eq!(fs::read(f.request.join("sub/keep")).unwrap(), b"local obstruction must survive\n");
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"chosen resolution\n");
    assert_eq!(f.blob(&new, "sub/keep"), b"zone changed this path\n");
    assert_eq!(f.blob(&new, "src/file"), b"chosen resolution\n");
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(r.zone.clone())); assert_one_rebase(f, &r.old, &new);
    assert!(!r.state.exists()); assert!(!f.lock_path().exists());
    // CLI read_pending_move -> Oplog's valued log: the durable tree is old tip + resolved paths, not the new tip.
    let expected_tree = crate::objects::tree(f, &r.old, &[("src/file", b"chosen resolution\n")], &[], &[]);
    assert_ne!(expected_tree, f.git.commit_info(&new).unwrap().tree);
    let before_read = f.observe();
    let pending = cli::read_pending_move(&f.git, &f.common, REQUEST, "join").unwrap().unwrap();
    assert_eq!((pending.old, pending.new, pending.folder_tree), (r.old.clone(), new.clone(), Some(expected_tree.clone())));
    assert_eq!(f.observe(), before_read, "reading the valued log must not write or move anything");
    let valued = oplog::read_valued_log(&f.common).unwrap();
    let folder_value = ("folder_tree".to_string(), expected_tree.as_str().to_string());
    // Until WO-join-9 lands, accept only its exact new shape or the exact existing shape (Orc #2426).
    for record in valued.records.iter().filter(|v| v.record.kind == "join-rebase") {
        assert!(record.values == vec![folder_value.clone()]
            || record.values == vec![folder_value.clone(), ("join_state".to_string(), state_before.clone())],
            "unexpected values: {:?}", record.values);
    }

    // Follow the refusal's actual next step, leaving the chosen resolution intact.
    f.write("sub/keep", b"keep\n");
    let start = f.records().len();
    let done = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(done.text("finished_move_from"), r.old.as_str());
    assert_eq!(done.text("finished_move_to"), new.as_str());
    assert!(!done.flag("saved_written"), "catch-up precedes save; old folder bytes are not new request work");
    // After catch-up, join fast-forwards the zone to the already-rebased request (run_join / S-72).
    assert_eq!(f.head(), new); assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(new.clone()));
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"chosen resolution\n");
    assert_eq!(fs::read(f.request.join("sub/keep")).unwrap(), b"zone changed this path\n");
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{}\n", f.git.commit_info(&new).unwrap().tree).as_bytes());
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty());
    let records = f.records();
    let first_snapshot = records[start..].iter().find(|r| r.kind == "snapshot" && r.phase == oplog::OpPhase::Done).unwrap();
    assert_eq!(f.blob(&first_snapshot.moves[0].new, "sub/keep"), b"keep\n");
    assert_eq!(f.blob(&first_snapshot.moves[0].new, "src/file"), b"chosen resolution\n");
    assert_eq!(f.blob(&snap, "sub/keep"), b"local obstruction must survive\n");
    assert_one_rebase(f, &r.old, &new); assert!(!r.state.exists()); assert!(!f.lock_path().exists());
    let before_repeat = f.observe();
    let repeat = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(repeat.text("finished_move_to"), new.as_str()); assert!(!repeat.flag("saved_written"));
    assert_eq!(f.observe(), before_repeat); assert!(!f.lock_path().exists());
}
