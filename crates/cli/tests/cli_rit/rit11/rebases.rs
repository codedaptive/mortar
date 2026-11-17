//! CLI undo/finish_pending_move/read_stalled_move/run_save -> real Join rebase, Oplog reverse and folder moves.
use super::fixture::*;
use crate::support::{REQUEST, SNAP, ZONE};
use std::fs;

#[test]
fn undo_rebase_moves_the_actual_folder_back_and_the_repeat_is_not_a_redo() {
    let w = World::new(); let (old, new, zone) = rebase(&w); let target = done(&w, "join-rebase").pop().unwrap();
    let r = ours(&w, &["undo"], 0, "undone"); let undo = assert_one_undo(&w, &target, &new, &old);
    assert_eq!(r.text("undo_op"), undo.op_id); assert_eq!(w.s.f.head(), old); assert_eq!(w.tip("core"), zone);
    index_at(&w, &w.s.f.request, &old); assert!(!w.s.f.request.join("src/upstream").exists());
    assert_eq!(fs::read(w.s.f.request.join("src/file")).unwrap(), b"topic work\n");
    let pending = cli::read_pending_move(&w.s.f.git, &w.s.f.common, REQUEST, "undo").unwrap().unwrap();
    assert_eq!(pending.cause, cli::MoveCause::Undo); assert_eq!((pending.old, pending.new, pending.folder_tree), (new, old, None));
    // S-144: the reverse folder move restored the branch's tree, so it is saved despite the newer snapshot.
    let before = observe(&w); let status = ours(&w, &["status"], 0, "saved");
    assert!(status.flag("saved")); assert_eq!(status.text("next"), "mortar join");
    assert!(status.is_null("stalled_from")); assert!(status.is_null("stalled_to")); assert_eq!(observe(&w), before);
    let files = w.folder_state(&w.s.f.request); let start = w.s.f.records().len();
    let repeat = ours(&w, &["undo"], 0, "already-done"); assert_eq!(repeat.text("undo_op"), undo.op_id);
    // Finishing the already-completed move writes exactly one snapshot op: its intent and done (two records).
    assert_eq!(w.folder_state(&w.s.f.request), files); assert_eq!(w.s.f.head(), undo.moves[0].new);
    let records = w.s.f.records(); assert_eq!(records.len() - start, 2, "exactly one snapshot's intent and done");
    assert_eq!(done(&w, "undo").len(), 1); assert!(records[start..].iter().all(|r| r.kind == "snapshot"));
    let before = observe(&w); ours(&w, &["undo"], 0, "already-done"); assert_eq!(observe(&w), before);
    assert!(!w.s.f.lock_path().exists());
}

#[test]
fn refused_reverse_move_has_undo_status_next_then_clear_and_undo_save_join_or_resolve_finishes() {
    for finisher in ["undo", "save", "join", "resolve"] {
        let w = World::new(); let (old, new, zone) = rebase(&w); let target = done(&w, "join-rebase").pop().unwrap();
        w.s.f.write("src/upstream", b"owner obstruction must survive\n"); let folder = w.folder_state(&w.s.f.request);
        let r = ours(&w, &["undo"], 1, "refused"); assert_eq!(r.text("owner"), "this request's folder");
        assert!(r.strings("lines").contains(&"  src/upstream".into()));
        let undo = assert_one_undo(&w, &target, &new, &old); assert_eq!(w.s.f.head(), old);
        assert_eq!(w.tip("core"), zone); assert_eq!(w.folder_state(&w.s.f.request), folder);
        let kept = ref_at(&w, SNAP).unwrap(); assert_eq!(w.s.f.blob(&kept, "src/upstream"), b"owner obstruction must survive\n");
        let before = observe(&w); let status = ours(&w, &["status"], 0, "stalled");
        assert_eq!(status.text("next"), "mortar undo"); assert_eq!(status.text("stalled_from"), new.as_str());
        assert_eq!(status.text("stalled_to"), old.as_str());
        assert!(status.strings("lines").contains(&format!("an undo's folder move is stalled: the undo put {REQUEST} back at {old}, the folder at {new}")));
        assert_eq!(observe(&w), before);
        let saves = done(&w, "save").len();
        let still = ours(&w, &["save", "Do not save a stale folder"], 1, "refused");
        assert!(still.strings("lines").contains(&"  src/upstream".into()));
        assert_eq!(w.s.f.head(), old); assert_eq!(done(&w, "save").len(), saves); assert_eq!(w.folder_state(&w.s.f.request), folder);
        assert_eq!(done(&w, "undo").len(), 1);
        let held = w.s.f.root.join("owner-obstruction"); fs::rename(w.s.f.request.join("src/upstream"), &held).unwrap();
        let words = if finisher == "save" { vec!["save", "No stale upstream"] } else { vec![finisher] };
        let outcome = match finisher { "undo" => "already-done", "save" => "unchanged", "join" => "rebased", "resolve" => "nothing-to-resolve", _ => unreachable!() };
        let report = ours(&w, &words, 0, outcome);
        if finisher == "undo" { assert_eq!(report.text("undo_op"), undo.op_id); }
        else if finisher != "resolve" {
            assert_eq!(report.text("finished_move_from"), new.as_str()); assert_eq!(report.text("finished_move_to"), old.as_str());
        }
        // With no paused state, resolve reports nothing-to-resolve after finishing; that report has no move fields.
        assert_eq!(done(&w, "undo").len(), 1); assert_eq!(done(&w, "save").len(), saves);
        assert_eq!(fs::read(&held).unwrap(), b"owner obstruction must survive\n");
        assert_eq!(w.s.f.blob(&kept, "src/upstream"), b"owner obstruction must survive\n");
        assert_eq!(fs::read(w.s.f.request.join("src/file")).unwrap(), b"topic work\n"); assert_eq!(ref_at(&w, ZONE), Some(zone.clone()));
        if finisher == "join" {
            assert_eq!(w.s.f.git.commit_info(&w.s.f.head()).unwrap().parents, vec![zone]);
            assert_eq!(fs::read(w.s.f.request.join("src/upstream")).unwrap(), b"upstream work\n"); assert!(!report.flag("saved_written"));
        } else { assert_eq!(w.s.f.head(), old); assert!(!w.s.f.request.join("src/upstream").exists()); }
        index_at(&w, &w.s.f.request, &w.s.f.head()); assert!(!w.s.f.lock_path().exists());
    }
}

#[test]
fn undo_finishes_an_existing_rebase_move_before_it_snapshots_and_reverses_the_branch() {
    let w = World::new(); let (old, rebased, zone) = w.s.f.pending_rebase();
    assert!(!w.s.f.request.join("src/upstream").exists()); index_at(&w, &w.s.f.request, &old);
    let target = done(&w, "join-rebase").pop().unwrap(); let r = ours(&w, &["undo"], 0, "undone");
    let snap = gitio::Oid::parse(r.text("snapshot")).unwrap();
    assert_eq!(w.s.f.blob(&snap, "src/upstream"), b"upstream work\n", "snapshot must follow forward catch-up, not record the stale folder");
    assert_eq!(w.s.f.blob(&snap, "src/file"), b"topic work\n");
    assert_one_undo(&w, &target, &rebased, &old); assert_eq!(w.s.f.head(), old); assert_eq!(w.tip("core"), zone);
    index_at(&w, &w.s.f.request, &old); assert!(!w.s.f.request.join("src/upstream").exists());
}

#[test]
fn an_obstructed_forward_pending_move_refuses_undo_before_the_undo_op_runs() {
    let w = World::new(); let (old, new, _) = w.s.f.pending_rebase();
    w.s.f.write("src/upstream", b"untracked obstruction\n"); let folder = w.folder_state(&w.s.f.request);
    let r = ours(&w, &["undo"], 1, "refused"); assert!(r.strings("lines").contains(&"  src/upstream".into()));
    assert!(done(&w, "undo").is_empty()); assert_eq!(w.s.f.head(), new); assert_eq!(w.folder_state(&w.s.f.request), folder);
    index_at(&w, &w.s.f.request, &old); let snap = ref_at(&w, SNAP).unwrap();
    assert_eq!(w.s.f.blob(&snap, "src/upstream"), b"untracked obstruction\n");
}
