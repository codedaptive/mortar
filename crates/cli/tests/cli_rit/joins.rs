use crate::support::*;
use std::fs;

#[test]
fn rit_join_saves_unsaved_work_then_fast_forwards_zone_and_repeat_is_inert() {
    let f = Fixture::new(); f.write("src/file", b"unsaved join work\n"); let start = f.records().len();
    let report = f.report(&["--json", "join"], "join", 0, "joined");
    let head = f.head(); assert!(report.flag("saved_written")); assert_eq!(report.text("saved_commit"), head.as_str());
    assert!(report.is_null("finished_move_from")); assert!(report.is_null("finished_move_to"));
    assert_eq!(report.text("next"), "wait for the zone's tests"); assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(head.clone()));
    let info = f.git.commit_info(&head).unwrap();
    assert!(info.message.starts_with("task: work saved at join\n\n")); assert_eq!(info.parents, vec![f.base.clone()]);
    assert_eq!(f.blob(&head, "src/file"), b"unsaved join work\n");
    let records = f.records(); let added = &records[start..]; assert_eq!(added.len(), 6);
    assert_eq!(added.iter().map(|r| r.kind.as_str()).collect::<Vec<_>>(), ["snapshot", "snapshot", "save", "save", "join-fast-forward", "join-fast-forward"]);
    assert!(added.iter().all(|r| r.actor == "core/task")); assert!(!f.lock_path().exists());
    let before = f.observe();
    let repeat = f.report(&["--json", "join"], "join", 0, "joined");
    assert!(!repeat.flag("saved_written")); assert_eq!(f.observe(), before); assert!(!f.lock_path().exists());
}

#[test]
fn rit_join_rebases_diverged_zone_moves_real_folder_and_reports_pending_move_even_when_already_finished() {
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(false);
    let old_info = f.git.commit_info(&old).unwrap();
    let report = f.report(&["--json", "join"], "join", 0, "rebased"); let new = f.head();
    assert_ne!(new, old); assert_eq!(report.text("from"), old.as_str()); assert_eq!(report.text("to"), new.as_str());
    assert_eq!(report.text("next"), "mr join"); assert!(!report.flag("saved_written"));
    let info = f.git.commit_info(&new).unwrap(); assert_eq!(info.parents, vec![zone.clone()]);
    assert_eq!(info.message, old_info.message); assert_eq!(info.author, old_info.author);
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(zone));
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"topic work\n");
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{}\n", info.tree).as_bytes());
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty());
    let again = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(again.text("finished_move_from"), old.as_str()); assert_eq!(again.text("finished_move_to"), new.as_str());
    assert!(!again.flag("saved_written"));
    let before = f.observe();
    let repeat = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(repeat.text("finished_move_to"), new.as_str()); assert_eq!(f.observe(), before);
    assert_eq!(f.records().iter().filter(|r| r.kind == "join-rebase" && r.phase == oplog::OpPhase::Done).count(), 1);
}

#[test]
fn rit_conflicting_join_lists_files_and_real_state_with_no_branch_or_folder_movement() {
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(true); let log = f.log();
    let report = f.report(&["--json", "join"], "join", 2, "conflicted");
    assert_eq!(report.text("next"), "mr resolve"); assert_eq!(report.text("commit"), old.as_str());
    assert_eq!(report.strings("files"), &["src/file"]); assert_eq!(f.head(), old);
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(zone)); assert_eq!(f.log(), log);
    let dir = f.local().git_dir(&f.request).unwrap();
    assert_eq!(report.text("state"), dir.join("mortar-join.json").to_str().unwrap());
    let state = join::paused_join(&dir).unwrap().unwrap(); assert_eq!(state.commit, old); assert_eq!(state.files, vec![std::path::PathBuf::from("src/file")]);
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"topic work\n"); assert!(!f.lock_path().exists());
}

#[test]
fn rit_join_arguments_are_usage_without_work_and_missing_zone_is_owned_refusal_with_lock_released() {
    let f = Fixture::new(); let before = f.observe();
    let usage = f.report(&["--json", "join", "unexpected"], "join", 4, "usage");
    assert!(usage.lines().contains("join takes no arguments")); assert_eq!(usage.text("next"), "mr join [--key <operation key>] [--json]");
    assert_eq!(f.observe(), before); assert!(!f.lock_path().exists());
    raw(&f.git, &["update-ref", "-d", ZONE]);
    let refused = f.report(&["--json", "join"], "join", 1, "refused");
    assert_eq!(refused.text("owner"), "the zone's coordinator"); assert!(refused.lines().contains("zone/core"));
    assert_eq!(f.head(), f.base); assert!(!f.lock_path().exists());
}

#[test]
fn rit_pending_folder_move_refuses_over_local_work_then_finishes_after_work_is_moved_aside() {
    let f = Fixture::new(); let (old, new, _) = f.pending_rebase();
    f.write("src/upstream", b"local obstruction must survive\n");
    let refused = f.report(&["--json", "join"], "join", 1, "refused");
    // The pending move's snapshot wrote the obstruction before the move was refused, so the refusal names that move.
    assert_eq!(refused.strings("lines"), &[
        "the snapshot ref moved as the work was saved first",
        "the folder could not move; these files stand in the way:", "  src/upstream",
        "the request branch already stands at the new tip; the next mr join, mr resolve, mr save or mr undo finishes the move first",
    ]);
    assert_eq!(refused.text("owner"), "this request's folder");
    assert!(refused.text("next").contains("move those files out of the way"));
    assert_eq!(f.head(), new); assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"local obstruction must survive\n");
    let preserved = f.git.read_ref(SNAP).unwrap().unwrap();
    assert_eq!(f.blob(&preserved, "src/upstream"), b"local obstruction must survive\n"); assert!(!f.lock_path().exists());
    fs::rename(f.request.join("src/upstream"), f.root.join("held-obstruction")).unwrap();
    let done = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(done.text("finished_move_from"), old.as_str()); assert_eq!(done.text("finished_move_to"), new.as_str());
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    assert_eq!(fs::read(f.root.join("held-obstruction")).unwrap(), b"local obstruction must survive\n");
    assert_eq!(f.head(), new); assert!(!f.lock_path().exists());
}
