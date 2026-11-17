//! CLI choose_undo/later_op_refusal -> real zone/actor history and actual resolved Join state, without fakes.
use super::fixture::*;
use crate::support::{raw, REQUEST, SNAP, ZONE};
use oplog::{KillPidProbe, LockName, Op, RefMove};
use std::fs;

#[test]
fn a_completed_join_is_refused_to_the_zone_owner_without_snapshot_or_folder_changes() {
    let w = World::new(); w.core(&[("src/file", b"joined work\n")]);
    let target = done(&w, "join-fast-forward").pop().unwrap(); let before = observe(&w);
    let r = ours(&w, &["undo"], 1, "refused");
    assert_eq!(r.strings("lines"), &[format!("op {} is a completed join: the zone owns it now", target.op_id), "nothing was changed".into()]);
    assert_eq!(r.text("owner"), "the zone's coordinator");
    assert_eq!(r.text("next"), format!("ask the zone's coordinator to back out op {}", target.op_id));
    assert_eq!(observe(&w), before); assert!(done(&w, "undo").is_empty());
}

#[test]
fn a_later_real_logged_actor_move_is_named_without_undoing_that_actors_work() {
    let w = World::new(); w.s.f.save_topic(); let target = done(&w, "save").pop().unwrap();
    let foreign = w.s.f.commit(&w.s.f.head(), &[("src/foreign", b"foreign work\n")], "another owner\n");
    let op = Op { op_id: "foreign-move".into(), key: "foreign-key".into(), kind: "save".into(), actor: "peer/owner".into(),
        reason: "foreign save".into(), at: 1700000000, moves: vec![RefMove { refname: REQUEST.into(), old: Some(w.s.f.head()), new: foreign.clone() }] };
    oplog::with_lock(&w.s.f.git, &w.s.f.common, &LockName::Join("core".into()), "foreign-lock", &KillPidProbe, || {
        oplog::recover(&w.s.f.git, &w.s.f.common).unwrap();
        assert!(matches!(oplog::apply_protected(&w.s.f.git, &w.s.f.common, &op, &[]).unwrap(), oplog::ApplyOutcome::Done(_)));
        Ok::<(), std::io::Error>(())
    }).unwrap();
    let folder = w.folder_state(&w.s.f.request); let zone = w.tip("core"); let remote = w.s.remote_refs();
    let r = ours(&w, &["undo"], 1, "refused");
    assert_eq!(r.strings("lines"), &[format!("op {} cannot be undone: {REQUEST} moved since, by op foreign-move of peer/owner", target.op_id), "nothing was undone; the snapshot taken first is kept".into()]);
    assert_eq!(r.text("owner"), "peer/owner"); assert_eq!(r.text("next"), format!("ask the zone's coordinator about {REQUEST}"));
    assert_eq!(w.s.f.head(), foreign); assert_eq!(w.tip("core"), zone); assert_eq!(w.s.remote_refs(), remote);
    assert_eq!(w.folder_state(&w.s.f.request), folder); assert!(done(&w, "undo").is_empty());
    assert_eq!(w.s.f.blob(&ref_at(&w, SNAP).unwrap(), "src/file"), b"topic work\n");
}

#[test]
fn resolve_undo_finishes_its_real_pending_folder_move_first_and_a_refused_move_undoes_nothing() {
    let w = World::new(); w.s.f.save_topic();
    let zone = w.s.f.commit(&w.s.f.base, &[("src/file", b"zone conflict\n"), ("sub/keep", b"zone changed keep\n")], "zone conflict\n");
    raw(&w.s.f.git, &["update-ref", ZONE, zone.as_str()]);
    ours(&w, &["join"], 2, "conflicted"); ours(&w, &["resolve"], 2, "files-written");
    w.s.f.write("src/file", b"chosen resolution\n"); w.s.f.write("sub/keep", b"owner obstruction\n");
    ours(&w, &["resolve"], 1, "refused"); let target = done(&w, "join-rebase").pop().unwrap();
    assert!(!w.s.f.local().git_dir(&w.s.f.request).unwrap().join("mortar-join.json").exists());
    let pending = cli::read_pending_move(&w.s.f.git, &w.s.f.common, REQUEST, "undo").unwrap().unwrap();
    assert!(pending.folder_tree.is_some());
    // S-143 (5) 4: undo finishes the resolve's pending folder move before oplog undo; sub/keep stands in its way.
    let (snap_before, before) = (ref_at(&w, SNAP), observe(&w)); let r = ours(&w, &["undo"], 1, "refused");
    assert_eq!(r.strings("lines"), &["the folder could not move; these files stand in the way:", "  sub/keep", "the request branch already stands at the new tip; the next mr join, mr resolve, mr save or mr undo finishes the move first"]);
    assert_eq!(r.text("owner"), "this request's folder"); assert_eq!(r.text("next"), "move those files out of the way, then mr join");
    assert!(done(&w, "undo").is_empty()); let new_tip = &target.moves.iter().find(|m| m.refname == REQUEST).unwrap().new;
    assert_eq!(ref_at(&w, REQUEST).as_ref(), Some(new_tip));
    assert!(!w.s.f.local().git_dir(&w.s.f.request).unwrap().join("mortar-join.json").exists());
    assert_eq!(fs::read(w.s.f.request.join("sub/keep")).unwrap(), b"owner obstruction\n");
    // decide_snapshot writes no commit for an unchanged tree: the snapshot resolve took is kept, the ref unchanged.
    let snap = ref_at(&w, SNAP).unwrap(); assert_eq!(Some(snap.clone()), snap_before);
    assert_eq!(w.s.f.blob(&snap, "sub/keep"), b"owner obstruction\n"); assert_eq!(observe(&w), before);
}

#[test]
fn nothing_to_undo_has_status_next_and_does_not_write_an_op_or_take_a_snapshot() {
    let w = World::new(); let before = observe(&w);
    // S-145: a pristine folder equals its branch tip but has no work of its own yet; neither join nor wait.
    let status = ours(&w, &["status"], 0, "saved"); assert!(status.flag("saved"));
    let r = ours(&w, &["undo"], 0, "nothing-to-undo");
    assert_eq!(r.strings("lines"), &["nothing to undo: no save, rebase or resolve of this folder is in the op log"]);
    assert_eq!(r.text("next"), "make your change, then mr save"); assert_eq!(r.text("next"), status.text("next"));
    assert_eq!(observe(&w), before); assert!(done(&w, "undo").is_empty());
}
