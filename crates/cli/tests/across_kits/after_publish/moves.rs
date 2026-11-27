//! after_publish/run_after_publish and zone_folder_move/bring_zone_folder joined to real Land, Join, Folders and Oplog, with acceptance step 1's check 14.
use super::fixture::World;
use crate::support::raw;
use std::fs;

#[test]
fn zone_b_made_before_r3_and_several_more_joins_gets_all_joined_bytes_after_publish() {
    let w = World::new(); let base_tree = w.s.f.git.commit_info(&w.s.f.base).unwrap().tree;
    let r3 = w.b_join("r3", &[("src/r3", b"r3's literal contribution\n")]);
    w.b_join("r4", &[("src/r4", b"four\n")]); w.b_join("r5", &[("src/r5", b"five\n")]);
    assert_eq!(w.s.f.git.index_tree(&w.b).unwrap(), base_tree); assert!(!w.b.join("src/r3").exists());
    w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish();
    let report = w.after(0, "after-published");
    assert_eq!(report.strings("moved"), &["core", "b"]); assert!(report.strings("conflicted").is_empty());
    assert_eq!(report.strings("folders_moved"), &[w.s.f.zone.display().to_string(), w.b.display().to_string()]);
    let tip = w.tip("b"); assert_ne!(tip, r3);
    w.folder_at(&w.b, &tip, &[("src/file", b"published\n"), ("src/r3", b"r3's literal contribution\n"), ("src/r4", b"four\n"), ("src/r5", b"five\n")]);
    w.folder_at(&w.s.f.zone, &origin, &[("src/file", b"published\n")]); w.assert_origin(&origin);
    let before = w.store(); let folders = (w.folder_state(&w.b), w.folder_state(&w.s.f.zone));
    let again = w.after(0, "after-published");
    assert_eq!(again.strings("current"), &["core", "b"]); assert!(again.strings("moved").is_empty());
    assert!(again.strings("folders_moved").is_empty()); assert_eq!(again.text("next"), "nothing to do");
    assert_eq!(w.store(), before); assert_eq!((w.folder_state(&w.b), w.folder_state(&w.s.f.zone)), folders);
}

#[test]
fn current_zones_catch_up_folders_without_writing_a_branch_op_or_changing_any_ref() {
    let w = World::new(); w.b_join("r3", &[("src/r3", b"three\n")]);
    w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish(); w.kit_after();
    assert!(!w.b.join("src/r3").exists()); assert_eq!(fs::read(w.s.f.zone.join("src/file")).unwrap(), b"base\n");
    let before = w.store(); let r = w.after(0, "after-published");
    assert_eq!(r.strings("current"), &["core", "b"]); assert!(r.strings("moved").is_empty());
    assert_eq!(r.strings("folders_moved").len(), 2); assert_eq!(w.store(), before);
    w.folder_at(&w.b, &w.tip("b"), &[("src/file", b"published\n"), ("src/r3", b"three\n")]);
    w.folder_at(&w.s.f.zone, &origin, &[("src/file", b"published\n")]);
}

#[test]
fn a_conflicted_zone_keeps_its_branch_but_its_lagging_folder_is_brought_to_that_branch_tip() {
    let w = World::new(); let b_tip = w.b_join("r3", &[("src/file", b"b's conflicting bytes\n")]);
    w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish();
    let r = w.after(0, "after-published");
    assert_eq!(r.strings("moved"), &["core"]); assert_eq!(r.strings("conflicted"), &["b"]);
    assert_eq!(w.tip("b"), b_tip); assert_eq!(r.strings("folders_moved").len(), 2);
    assert!(r.strings("lines").contains(&format!("zone b left in place at {b_tip}: replaying {b_tip} onto origin conflicts in:")));
    assert!(r.strings("lines").contains(&"src/file".into()));
    assert_eq!(r.text("next"), "zone b conflicts with origin: its coordinator resolves the conflict; then repeat the same command: mr after-publish core --move b --actor <name>");
    w.folder_at(&w.b, &b_tip, &[("src/file", b"b's conflicting bytes\n")]);
    w.folder_at(&w.s.f.zone, &origin, &[("src/file", b"published\n")]); w.assert_origin(&origin);
    assert!(raw(&w.git(&w.b), &["ls-files", "-u"]).is_empty());
}

#[test]
fn refused_pre_replay_folder_is_found_in_done_op_history_on_a_current_repeat() {
    let w = World::new(); let old = w.b_join("r3", &[("src/r3", b"three\n")]);
    // Put the zone folder at its actual pre-replay tip; after_publish must later find this off-line held value.
    raw(&w.git(&w.b), &["read-tree", "-m", "-u", w.s.f.base.as_str(), old.as_str()]);
    crate::support::write(&w.b, "src/file", b"owner's obstruction\n");
    w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish(); let preserved = w.folder_state(&w.b);
    let refused = w.after(3, "folder-not-moved"); let tip = w.tip("b");
    assert_eq!(refused.strings("moved"), &["core", "b"]); assert_eq!(w.folder_state(&w.b), preserved);
    assert_ne!(w.s.f.git.merge_base(&old, &tip).unwrap(), Some(old.clone()));
    assert!(w.s.f.records().iter().any(|record| record.phase == oplog::OpPhase::Done && record.moves.iter().any(|m| m.refname == "refs/heads/zone/b" && m.old.as_ref() == Some(&old) && m.new == tip)));
    let saved = w.s.f.root.join("owner-obstruction"); fs::rename(w.b.join("src/file"), &saved).unwrap();
    crate::support::write(&w.b, "src/file", b"base\n"); let before = w.store();
    let repeated = w.after(0, "after-published");
    assert_eq!(repeated.strings("current"), &["core", "b"]); assert!(repeated.strings("moved").is_empty());
    assert_eq!(repeated.strings("folders_moved"), &[w.b.display().to_string()]); assert_eq!(w.store(), before);
    assert_eq!(fs::read(saved).unwrap(), b"owner's obstruction\n");
    w.folder_at(&w.b, &tip, &[("src/file", b"published\n"), ("src/r3", b"three\n")]); w.assert_origin(&origin);
}

#[test]
fn held_value_r1_whole_tree_revert_moves_to_tip() {
    held_value_r1_revert(true);
}

#[test]
fn held_value_r1_partial_revert_is_refused() {
    held_value_r1_revert(false);
}

fn held_value_r1_revert(whole: bool) {
    let w = World::new(); let held = w.core(&[("src/file", b"held value\n")]);
    w.core(&[("src/file", b"latest value\n"), ("src/later", b"later join\n")]);
    let (_, origin) = w.publish(); w.after(0, "after-published");
    assert!(w.s.f.records().iter().any(|r| r.phase == oplog::OpPhase::Done && r.moves.iter().any(|m| m.refname == "refs/heads/zone/core" && m.new == held)));
    if whole { raw(&w.git(&w.s.f.zone), &["read-tree", "-m", "-u", origin.as_str(), held.as_str()]); }
    else { raw(&w.git(&w.s.f.zone), &["restore", "--source", held.as_str(), "--staged", "--worktree", "--", "src/file"]); }
    let before = w.store(); let folder = w.folder_state(&w.s.f.zone);
    if whole {
        assert_eq!(fs::read(w.s.f.zone.join("src/file")).unwrap(), b"held value\n");
        let r = w.after(0, "after-published"); assert_eq!(r.strings("folders_moved"), &[w.s.f.zone.display().to_string()]);
        w.folder_at(&w.s.f.zone, &origin, &[("src/file", b"latest value\n"), ("src/later", b"later join\n")]);
    } else {
        let r = w.after(3, "folder-not-moved"); assert!(r.strings("lines").contains(&format!("the zone folder {} was not brought to {origin}: its index matches no commit zone core has held", w.s.f.zone.display())));
        assert_eq!(w.folder_state(&w.s.f.zone), folder);
    }
    assert_eq!(w.s.f.blob(&held, "src/file"), b"held value\n"); assert_eq!(w.store(), before); w.assert_origin(&origin);
}
