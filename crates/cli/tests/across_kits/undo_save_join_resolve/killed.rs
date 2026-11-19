//! Actual CLI SIGKILL after the undo/rebase Done record and before its index/folder step, then plain repeat/save.
//! The Git boundary and durable records/files decide the result, never elapsed time or a fabricated kit answer.
use super::fixture::*;
use crate::support::SNAP;
use std::fs;

#[test]
fn undo_killed_before_set_index_or_reverse_folder_move_repeats_once_without_redo() {
    for step in ["save", "rebase"] {
        let w = World::new();
        let (old, new, target, flag) = if step == "save" {
            let saved = w.s.f.save_topic(); (w.s.f.base.clone(), saved, done(&w, "save").pop().unwrap(), "--end-of-options")
        } else {
            let (old, new, _) = rebase(&w); (old, new, done(&w, "join-rebase").pop().unwrap(), "--no-sparse-checkout")
        };
        let before_index = clean_index_bytes(&w); let before_files = w.folder_state(&w.s.f.request).1;
        kill_before_step(&w, "undo", "undo", flag);
        let undo = assert_one_undo(&w, &target, &new, &old);
        assert_eq!(w.s.f.head(), old); assert_eq!(clean_index_bytes(&w), before_index);
        assert_eq!(w.folder_state(&w.s.f.request).1, before_files);
        assert_eq!(oplog::open_intent(&w.s.f.common, &undo.op_id).unwrap(), None);
        assert!(w.s.f.lock_path().exists(), "the dead holder is left for locked_act recovery");
        if step == "rebase" {
            let before = observe(&w); let status = ours(&w, &["status"], 0, "stalled");
            assert_eq!(status.text("next"), "mr undo"); assert_eq!(status.text("stalled_from"), new.as_str());
            assert_eq!(status.text("stalled_to"), old.as_str()); assert_eq!(observe(&w), before);
        }
        let r = ours(&w, &["undo"], 0, "already-done");
        assert_eq!(r.text("undo_op"), undo.op_id); assert_eq!(r.text("undone_op"), target.op_id);
        assert_eq!(w.s.f.head(), old); index_at(&w, &w.s.f.request, &old); assert!(!w.s.f.lock_path().exists());
        assert_eq!(fs::read(w.s.f.request.join("src/file")).unwrap(), b"topic work\n");
        if step == "save" { assert_eq!(w.folder_state(&w.s.f.request).1, before_files); }
        else { assert!(!w.s.f.request.join("src/upstream").exists()); }
        assert_eq!(done(&w, "undo").len(), 1);
        let files = w.folder_state(&w.s.f.request); let start = w.s.f.records().len();
        ours(&w, &["undo"], 0, "already-done");
        assert_eq!(w.s.f.head(), old); assert_eq!(w.folder_state(&w.s.f.request), files); assert_eq!(done(&w, "undo").len(), 1);
        // This repeat writes nothing after save undo; after a reverse move, one snapshot op (intent + done).
        let records = w.s.f.records();
        assert_eq!(records.len() - start, if step == "rebase" { 2 } else { 0 }, "at most one snapshot's intent and done");
        assert!(records[start..].iter().all(|r| r.kind == "snapshot"));
        let before = observe(&w); ours(&w, &["undo"], 0, "already-done"); assert_eq!(observe(&w), before);
    }
}

#[test]
fn save_after_a_real_killed_join_finishes_the_forward_move_before_saving_any_stale_bytes() {
    let w = World::new(); let old = w.s.f.save_topic(); let zone = w.s.f.advance_zone(false);
    let saves = done(&w, "save").len(); kill_before_step(&w, "join", "join-rebase", "--no-sparse-checkout");
    let new = w.s.f.head(); assert_ne!(new, old); index_at(&w, &w.s.f.request, &old);
    assert!(!w.s.f.request.join("src/upstream").exists());
    let report = ours(&w, &["save", "Do not commit the stale folder"], 0, "unchanged");
    assert_eq!(report.text("finished_move_from"), old.as_str()); assert_eq!(report.text("finished_move_to"), new.as_str());
    assert_eq!(report.text("commit"), new.as_str()); assert!(!report.flag("written")); assert_eq!(w.s.f.head(), new);
    assert_eq!(w.tip("core"), zone); assert_eq!(done(&w, "save").len(), saves); assert_eq!(done(&w, "join-rebase").len(), 1);
    index_at(&w, &w.s.f.request, &new); assert_eq!(fs::read(w.s.f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    assert_eq!(fs::read(w.s.f.request.join("src/file")).unwrap(), b"topic work\n");
    let snap = ref_at(&w, SNAP).unwrap(); assert_eq!(w.s.f.blob(&snap, "src/upstream"), b"upstream work\n");
    assert_eq!(w.s.f.last_move().record.kind, "join-rebase");
    let status = w.s.f.status(); assert!(status.is_null("stalled_from") && status.is_null("stalled_to"));
    assert!(!w.s.f.lock_path().exists());
    let before = observe(&w); ours(&w, &["save", "Do not commit the stale folder"], 0, "unchanged"); assert_eq!(observe(&w), before);
}
