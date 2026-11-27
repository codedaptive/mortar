//! CLI undo/finish_pending_move/read_stalled_move/run_save/run_join -> real Join rebase, Oplog reverse and folder moves.
use super::fixture::*;
use crate::support::{raw, REQUEST, SNAP, ZONE};
use std::fs;

#[test]
fn undo_rebase_moves_the_actual_folder_back_and_the_repeat_is_not_a_redo() {
    let w = World::new(); let (old, new, zone) = rebase(&w); let target = done(&w, "join-rebase").pop().unwrap();
    let r = ours(&w, &["undo"], 0, "undone"); let undo = assert_one_undo(&w, &target, &new, &old);
    assert_eq!(r.text("undo_op"), undo.op_id); assert_eq!(w.s.f.head(), old); assert_eq!(w.tip("core"), zone);
    index_at(&w, &w.s.f.request, &old); assert!(!w.s.f.request.join("src/upstream").exists());
    assert_eq!(fs::read(w.s.f.request.join("src/file")).unwrap(), b"topic work\n");
    let moved = w.s.f.last_move(); assert_eq!(moved.record.kind, "undo");
    let movement = moved.record.moves.iter().find(|m| m.refname == REQUEST).unwrap();
    assert_eq!(movement.old.as_ref(), Some(&new)); assert_eq!(movement.new, old);
    assert!(moved.values.iter().all(|(key, _)| key != "folder_tree"));
    // The reverse folder move restored the branch's tree, so it is saved despite the newer snapshot.
    let before = observe(&w); let status = ours(&w, &["status"], 0, "saved");
    assert!(status.flag("saved")); assert_eq!(status.text("next"), "mr join");
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
fn refused_reverse_move_has_undo_status_next_then_clear_and_undo_finishes() {
    refused_reverse_move_then_clear_and("undo");
}

#[test]
fn refused_reverse_move_has_undo_status_next_then_clear_and_save_finishes() {
    refused_reverse_move_then_clear_and("save");
}

#[test]
fn refused_reverse_move_has_undo_status_next_then_clear_and_join_finishes() {
    refused_reverse_move_then_clear_and("join");
}

#[test]
fn refused_reverse_move_has_undo_status_next_then_clear_and_resolve_finishes() {
    refused_reverse_move_then_clear_and("resolve");
}

fn refused_reverse_move_then_clear_and(finisher: &str) {
    let w = World::new(); let (old, new, zone) = rebase(&w); let target = done(&w, "join-rebase").pop().unwrap();
    w.s.f.write("src/upstream", b"owner obstruction must survive\n"); let folder = w.folder_state(&w.s.f.request);
    let r = ours(&w, &["undo"], 3, "refused"); assert_eq!(r.text("owner"), "this request's folder");
    assert!(r.strings("lines").contains(&"  src/upstream".into()));
    let undo = assert_one_undo(&w, &target, &new, &old); assert_eq!(w.s.f.head(), old);
    assert_eq!(w.tip("core"), zone); assert_eq!(w.folder_state(&w.s.f.request), folder);
    let kept = ref_at(&w, SNAP).unwrap(); assert_eq!(w.s.f.blob(&kept, "src/upstream"), b"owner obstruction must survive\n");
    let before = observe(&w); let status = ours(&w, &["status"], 0, "stalled");
    assert_eq!(status.text("next"), "mr undo"); assert_eq!(status.text("stalled_from"), new.as_str());
    assert_eq!(status.text("stalled_to"), old.as_str());
    assert!(status.strings("lines").contains(&format!("an undo's folder move is stalled: the undo put {REQUEST} back at {old}, the folder at {new}")));
    assert_eq!(observe(&w), before);
    let saves = done(&w, "save").len();
    let still = ours(&w, &["save", "Do not save a stale folder"], 3, "refused");
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
    // The finished move's snapshot wrote (the obstruction is gone from the folder, still in the snapshot ref): save,
    // join and resolve's nothing-to-resolve name it; undo names no moves.
    let named = "the snapshot ref moved as the work was saved first".to_string();
    let stands = format!("the folder stands where the undo put the branch back: {new} -> {old}");
    let expected = match finisher {
        "undo" => vec![format!("op {} was already undone by op {}", target.op_id, undo.op_id), "the folder stands at the tip before the rebase".to_string()],
        "save" => vec![stands, named, format!("nothing changed since the last save; no save commit written (HEAD is {old})")],
        "join" => vec![
            stands, "no unsaved work".to_string(),
            format!("rebased: {REQUEST} moved from {old} to {}, replayed onto the zone; the folder moved with it; {named}", w.s.f.head()),
            "the unit tests must run again".to_string(),
        ],
        "resolve" => vec![named, "no join is paused in this folder; nothing to resolve".to_string()],
        _ => unreachable!(),
    };
    assert_eq!(report.strings("lines"), expected.as_slice());
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
    let r = ours(&w, &["undo"], 3, "refused");
    // The pending move's snapshot wrote the obstruction (below), but undo names no moves: no moved line.
    assert_eq!(r.strings("lines"), &[
        "the folder could not move; these files stand in the way:", "  src/upstream",
        "the request branch already stands at the new tip; the next mr join, mr resolve, mr save or mr undo finishes the move first",
    ]);
    assert!(done(&w, "undo").is_empty()); assert_eq!(w.s.f.head(), new); assert_eq!(w.folder_state(&w.s.f.request), folder);
    index_at(&w, &w.s.f.request, &old); let snap = ref_at(&w, SNAP).unwrap();
    assert_eq!(w.s.f.blob(&snap, "src/upstream"), b"untracked obstruction\n");
}

#[test]
fn join_saving_unsaved_work_then_rebasing_names_the_save_and_snapshot_moves_when_a_build_file_blocks_the_folder_move() {
    let w = World::new(); let f = &w.s.f; let snap_before = ref_at(&w, SNAP);
    // The zone tracks a file under the fixture's build dir "d/"; the folder's own copy is excluded from the save, so it
    // stays untracked and stands in the way of the folder move after the rebase.
    let zone = f.commit(&f.base, &[("src/d/out", b"zone build output\n")], "zone adds build output\n");
    raw(&f.git, &["update-ref", ZONE, zone.as_str()]);
    f.write("src/file", b"unsaved join work\n"); f.write("src/d/out", b"local build output must survive\n");
    let r = ours(&w, &["join"], 3, "refused");
    let rebase = done(&w, "join-rebase").pop().unwrap();
    let movement = rebase.moves.iter().find(|m| m.refname == REQUEST).unwrap();
    let (old, new) = (movement.old.clone().unwrap(), movement.new.clone());
    // The join's save wrote the unsaved work on the base, and the rebase replayed it onto the zone.
    assert_eq!(f.git.commit_info(&old).unwrap().parents, vec![f.base.clone()]);
    assert_eq!(f.blob(&old, "src/file"), b"unsaved join work\n");
    assert_eq!(f.git.commit_info(&new).unwrap().parents, vec![zone.clone()]); assert_eq!(f.head(), new);
    let snap = ref_at(&w, SNAP).unwrap(); assert_ne!(Some(snap.clone()), snap_before);
    assert_eq!(f.blob(&snap, "src/file"), b"unsaved join work\n");
    assert_eq!(r.text("owner"), "this request's folder");
    assert_eq!(r.text("next"), "move those files out of the way, then mr join");
    assert_eq!(r.strings("lines"), &[
        format!("{REQUEST} and the snapshot ref moved as the work was saved first"),
        format!("rebased: {REQUEST} moved from {old} to {new}, replayed onto the zone"),
        "the folder could not move; these files stand in the way:".to_string(), "  src/d/out".to_string(),
        "the request branch already stands at the new tip; the next mr join, mr resolve, mr save or mr undo finishes the move first".to_string(),
    ]);
    assert_eq!(fs::read(f.request.join("src/d/out")).unwrap(), b"local build output must survive\n");
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"unsaved join work\n");
    index_at(&w, &f.request, &old); assert_eq!(ref_at(&w, ZONE), Some(zone)); assert!(!f.lock_path().exists());
}
