//! CLI run_overlap/read_named_zones -> real folder records, Snapshot tips, Join fork points and overlap paths.
use super::fixture::*;
use crate::{json::check, support::{raw, write, REQUEST, SNAP, ZONE}};
use std::fs;

#[test]
fn overlap_uses_unsaved_snapshot_tips_without_zone_folder_and_is_read_only_even_under_a_join_lock() {
    let w = World::new(); let peer = w.request("core", "peer");
    w.s.f.write("src/file", b"task unsaved\n"); write(&peer, "src/file", b"peer unsaved\n");
    for path in [&w.s.f.request, &peer] { run(&w, path, &["snapshot"], 0, "snapshotted"); }
    assert_eq!(ref_at(&w, REQUEST), Some(w.s.f.base.clone()));
    assert_eq!(ref_at(&w, "refs/heads/req/core/peer"), Some(w.s.f.base.clone()));
    assert_ne!(ref_at(&w, SNAP), ref_at(&w, REQUEST));
    raw(&w.s.f.git, &["worktree", "remove", w.s.f.zone.to_str().unwrap()]);
    let holder = oplog::LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "overlap-needs-no-lock".into(), since: 1700000000 };
    let lock = oplog::acquire_lock(&w.s.f.common, &oplog::LockName::Join("core".into()), &holder, &oplog::KillPidProbe).unwrap();
    let before = observe(&w); let r = app(&w, &["overlap", "core"], 0, "overlaps");
    assert_eq!(r.strings("pairs"), &["peer task"]);
    assert_eq!(r.strings("lines"), &["requests peer and task both change: src/file"]);
    assert_eq!(r.text("next"), "nothing to do"); assert_eq!(observe(&w), before); assert!(lock.path.exists());
    oplog::release_lock(&lock).unwrap();
}

#[test]
fn disjoint_saved_requests_give_no_overlap_and_preserve_every_folder_and_ref() {
    let w = World::new(); let peer = w.request("core", "peer");
    w.s.f.write("src/ours", b"ours\n"); write(&peer, "src/theirs", b"theirs\n");
    saved(&w, &w.s.f.request, "Save ours"); saved(&w, &peer, "Save theirs");
    let before = observe(&w); let r = app(&w, &["overlap", "core"], 0, "no-overlap");
    assert!(r.strings("pairs").is_empty());
    assert_eq!(r.strings("lines"), &["no two open requests of zone core change the same path"]);
    assert_eq!(observe(&w), before);
}

#[test]
fn a_real_rebase_excludes_zone_only_paths_instead_of_using_the_original_folder_cut() {
    let w = World::new(); let peer = w.request("core", "peer");
    w.s.f.write("src/file", b"task topic\n"); saved(&w, &w.s.f.request, "Save topic");
    write(&peer, "src/upstream", b"peer owns this path\n"); saved(&w, &peer, "Save peer");
    let zone = w.s.f.advance_zone(false);
    check(w.s.f.command(&w.s.f.request, &["--json", "join"]).output().unwrap(), "join", 0, "rebased");
    assert_eq!(w.s.f.git.commit_info(&w.s.f.head()).unwrap().parents, vec![zone.clone()]);
    run(&w, &w.s.f.request, &["snapshot"], 0, "snapshotted");
    let task_tip = ref_at(&w, SNAP).unwrap();
    assert_eq!(w.s.f.git.commit_info(&w.s.f.head()).unwrap().parents, vec![zone]);
    assert_eq!(w.s.f.blob(&task_tip, "src/upstream"), b"upstream work\n");
    let dir = w.s.f.git.git_dir(&w.s.f.request).unwrap();
    assert_eq!(folders::make::read_folder_record(&dir).unwrap().unwrap().base, w.s.f.base);
    let before = observe(&w); let r = app(&w, &["overlap", "core"], 0, "no-overlap");
    assert!(r.strings("pairs").is_empty()); assert_eq!(observe(&w), before);
}

#[test]
fn missing_zone_unrelated_request_and_corrupt_record_fail_under_overlap_before_any_mutation() {
    for kind in ["missing-zone", "unrelated", "bad-record"] {
        let w = World::new(); let peer = w.request("core", "victim");
        let expected = match kind {
            "missing-zone" => { raw(&w.s.f.git, &["update-ref", "-d", ZONE]); "refs/heads/zone/core does not exist".to_string() }
            "unrelated" => {
                let tree = w.s.f.git.commit_info(&w.s.f.base).unwrap().tree;
                let root = w.s.f.git.commit_tree(&tree, &[], "unrelated root\n", None, None).unwrap();
                raw(&w.s.f.git, &["update-ref", "refs/heads/req/core/victim", root.as_str()]);
                "the request victim shares no commit with zone core".to_string()
            }
            "bad-record" => {
                let dir = w.s.f.git.git_dir(&peer).unwrap(); fs::write(folders::record::record_path(&dir), b"not a record\n").unwrap();
                format!("the folder record of {} cannot be read", peer.display())
            }
            _ => unreachable!(),
        };
        let before = observe(&w); let r = app(&w, &["overlap", "core"], 5, "failed");
        assert_eq!(r.strings("lines"), &[expected, "nothing half-done is left".into()]);
        assert_eq!(r.text("next"), "repeat the same command"); assert_eq!(observe(&w), before);
    }
}
