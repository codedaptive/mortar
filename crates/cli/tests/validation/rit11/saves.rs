//! CLI run_save/run_undo -> real Snapshot save, Oplog reverse, Gitio index and Folders attempt isolation.
use super::fixture::*;
use crate::support::{write, REQUEST, SNAP};
use gitio::Oid;
use std::fs;

#[test]
fn undo_save_keeps_all_file_bytes_sets_the_restored_index_and_repeats_without_redo_or_another_op() {
    let w = World::new(); let saved = w.s.f.save_topic(); let target = done(&w, "save").pop().unwrap();
    w.s.f.write("src/file", b"later unsaved work\n"); w.s.f.write("src/new", b"new unsaved file\n");
    let before_files = w.folder_state(&w.s.f.request).1; let zone = w.tip("core"); let remote = w.s.remote_refs();
    let r = ours(&w, &["undo", "--key", "holder-key-not-op-key"], 0, "undone");
    let undo = assert_one_undo(&w, &target, &saved, &w.s.f.base);
    assert_eq!(r.text("undone_op"), target.op_id); assert_eq!(r.text("undone_kind"), "save");
    assert_eq!(r.text("undo_op"), undo.op_id); assert_eq!(r.text("next"), "mr status");
    let snap = Oid::parse(r.text("snapshot")).unwrap();
    assert_eq!(r.strings("lines"), &[format!("undid save op {}", target.op_id), format!("the undo is op {}", undo.op_id),
        format!("the folder as it stood is kept in snapshot commit {snap}"), "the files are untouched; the work shows as unsaved".into()]);
    assert_eq!(w.s.f.head(), w.s.f.base); index_at(&w, &w.s.f.request, &w.s.f.base);
    assert_eq!(w.folder_state(&w.s.f.request).1, before_files);
    assert_eq!(w.s.f.blob(&snap, "src/file"), b"later unsaved work\n"); assert_eq!(w.s.f.blob(&snap, "src/new"), b"new unsaved file\n");
    assert_eq!(w.tip("core"), zone); assert_eq!(w.s.remote_refs(), remote); assert!(!w.s.f.lock_path().exists());
    let before = observe(&w); let status = ours(&w, &["status"], 0, "unsaved");
    assert!(!status.flag("saved")); assert_eq!(status.text("next"), "mr save \"<sentence>\""); assert_eq!(observe(&w), before);
    let repeat = ours(&w, &["undo"], 0, "already-done");
    assert_eq!(repeat.text("undo_op"), undo.op_id); assert_eq!(repeat.text("undone_op"), target.op_id);
    assert_eq!(observe(&w), before); assert_eq!(done(&w, "undo").len(), 1);
}

#[test]
fn pre_undo_snapshot_ref_can_be_restored_by_the_real_restore_verb_after_later_work() {
    let w = World::new(); w.s.f.save_topic(); w.s.f.write("src/file", b"kept before undo\n");
    let r = ours(&w, &["undo"], 0, "undone"); let snap = Oid::parse(r.text("snapshot")).unwrap();
    let kept = done(&w, "snapshot").into_iter().find(|r| r.moves.iter().any(|m| m.refname == SNAP && m.new == snap)).unwrap();
    w.s.f.write("src/file", b"later saved work\n"); ours(&w, &["save", "Save later work"], 0, "saved");
    assert_ne!(ref_at(&w, SNAP), Some(snap.clone())); let branch = ref_at(&w, REQUEST); let folder = w.folder_state(&w.s.f.request);
    w.s.app(&["restore", &kept.op_id, SNAP], 0, "restored");
    assert_eq!(ref_at(&w, SNAP), Some(snap.clone())); assert_eq!(w.s.f.blob(&snap, "src/file"), b"kept before undo\n");
    assert_eq!(ref_at(&w, REQUEST), branch); assert_eq!(w.folder_state(&w.s.f.request), folder);
    let before = observe(&w); w.s.app(&["restore", &kept.op_id, SNAP], 0, "already-done"); assert_eq!(observe(&w), before);
}

#[test]
fn request_and_each_attempt_undo_only_their_own_save_and_snapshot_scope() {
    let w = World::new(); let request = w.s.f.save_topic();
    let attempts = w.s.f.root.join("attempts");
    w.s.run(&w.s.f.request, &["folder", "attempts", "2", attempts.to_str().unwrap()], 0, "made");
    let a = attempts.join("try-1"); let b = attempts.join("try-2");
    for (path, bytes) in [(&a, b"first attempt\n".as_slice()), (&b, b"second attempt\n".as_slice())] {
        write(path, "src/file", bytes); agent(&w, path, &["save", "Save attempt"], 0, "saved");
    }
    let b_branch = ref_at(&w, "refs/heads/try/core/task/2"); let b_snap = ref_at(&w, "refs/mortar/trysnap/core/task/2");
    let request_snap = ref_at(&w, SNAP);
    agent(&w, &a, &["undo"], 0, "undone");
    assert_eq!(ref_at(&w, "refs/heads/try/core/task/1"), Some(request.clone())); index_at(&w, &a, &request);
    assert_eq!(fs::read(a.join("src/file")).unwrap(), b"first attempt\n");
    assert_eq!(w.s.f.head(), request); assert_eq!(ref_at(&w, SNAP), request_snap);
    assert_eq!(ref_at(&w, "refs/heads/try/core/task/2"), b_branch); assert_eq!(ref_at(&w, "refs/mortar/trysnap/core/task/2"), b_snap);
    let attempts_before = (w.folder_state(&a), w.folder_state(&b), ref_at(&w, "refs/mortar/trysnap/core/task/1"));
    ours(&w, &["undo"], 0, "undone"); assert_eq!(w.s.f.head(), w.s.f.base);
    assert_eq!((w.folder_state(&a), w.folder_state(&b), ref_at(&w, "refs/mortar/trysnap/core/task/1")), attempts_before);
    assert_eq!(ref_at(&w, "refs/heads/try/core/task/2"), b_branch); assert_eq!(ref_at(&w, "refs/mortar/trysnap/core/task/2"), b_snap);
    assert_eq!(done(&w, "undo").iter().map(|r| r.moves[0].refname.as_str()).collect::<Vec<_>>(), vec!["refs/heads/try/core/task/1", REQUEST]);
    let before = observe(&w); agent(&w, &a, &["undo"], 0, "already-done"); ours(&w, &["undo"], 0, "already-done"); assert_eq!(observe(&w), before);
}
