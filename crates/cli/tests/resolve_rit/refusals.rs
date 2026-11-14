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
    assert_eq!(report.text("next"), "mortar resolve [--key <operation key>] [--json]");
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
            assert!(report.lines().contains(&format!("this is attempt folder {k} of request task: mortar {verb} runs in the request's own folder")));
            assert!(report.lines().contains("nothing ran"));
            assert_eq!(report.text("next"), "keep or drop the attempts, then mortar join in the request's folder");
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
    let report = f.report(&["--json", "resolve"], "resolve", 1, "refused"); let new = f.head();
    assert_ne!(new, r.old); assert_eq!(report.text("owner"), "this request's folder");
    assert!(report.lines().contains("edits on paths the zone changed stand in the way"));
    assert!(report.lines().contains("  sub/keep"));
    let snap = f.git.read_ref(SNAP).unwrap().unwrap();
    assert!(report.lines().contains(&format!("they are kept in {SNAP} at {snap}")));
    assert_eq!(report.text("next"), "put those files back to the folder's version, then mortar join");
    assert_eq!(f.blob(&snap, "sub/keep"), b"local obstruction must survive\n");
    assert_eq!(f.blob(&snap, "src/file"), b"chosen resolution\n");
    assert_eq!(fs::read(f.request.join("sub/keep")).unwrap(), b"local obstruction must survive\n");
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"chosen resolution\n");
    assert_eq!(f.blob(&new, "sub/keep"), b"zone changed this path\n");
    assert_eq!(f.blob(&new, "src/file"), b"chosen resolution\n");
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(r.zone.clone())); assert_one_rebase(f, &r.old, &new);
    assert!(!r.state.exists()); assert!(!f.lock_path().exists());
    // No retry/catch-up claim: the op's durable folder_tree is still join3e's explicitly deferred work.
}
