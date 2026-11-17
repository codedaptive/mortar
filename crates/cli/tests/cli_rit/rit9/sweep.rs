//! CLI run_sweep -> real Folders ref choice/deletion and build sweep -> Oplog/Gitio, including S-141 policy.
use super::fixture::*;
use crate::support::{raw, write};
use oplog::{OpPhase, OpResult};
use std::fs;

#[test]
fn sweep_deletes_extra_closed_snapshot_and_removed_zone_replay_refs_in_one_actor_op_but_keeps_live_zones() {
    let w = World::new(); let extra = "refs/t3/checkpoint";
    let closed = "refs/mortar/snap/core/closed"; let gone = "refs/mortar/replayed/gone/source";
    let core = "refs/mortar/replayed/core/source"; let b = "refs/mortar/replayed/b/source";
    for name in [extra, closed, gone, core, b] { raw(&w.s.f.git, &["update-ref", name, w.s.f.base.as_str()]); }
    let start = w.s.f.records().len(); let remote = w.s.remote_refs();
    let files = (w.folder_state(&w.s.f.request), w.folder_state(&w.s.f.zone), w.folder_state(&w.b));
    let r = app(&w, &["sweep", "--ref", extra, "--key", "holder-only-sweep"], 0, "swept");
    assert_eq!(r.strings("deleted"), &[closed, gone, extra]); assert!(r.strings("removed").is_empty()); assert!(r.is_null("freed_bytes"));
    assert_eq!(r.strings("lines"), &[format!("deleted {closed}"), format!("deleted {gone}"), format!("deleted {extra}"), "build dirs not swept: no --idle-days".into()]);
    for name in [closed, gone, extra] { assert_eq!(ref_at(&w, name), None); }
    for name in [core, b] { assert_eq!(ref_at(&w, name), Some(w.s.f.base.clone())); }
    let records = w.s.f.records(); let added = &records[start..]; assert_eq!(added.len(), 2);
    assert_eq!(added.iter().map(|r| r.phase).collect::<Vec<_>>(), vec![OpPhase::Intent, OpPhase::Done]);
    let done = &added[1]; assert_eq!(done.kind, "folder-sweep-refs"); assert_eq!(done.actor, "rit9");
    assert_eq!(done.reason, "sweep 3 refs"); assert_eq!(done.result, Some(OpResult::Moved));
    assert_eq!(done.key, format!("folders:sweep-refs:{}", done.op_id)); assert_ne!(done.key, "holder-only-sweep");
    assert_eq!(done.moves.iter().map(|m| m.refname.as_str()).collect::<Vec<_>>(), vec![closed, gone, extra]);
    assert!(done.moves.iter().all(|m| m.old == Some(w.s.f.base.clone()) && m.new.as_str() == "0000000000000000000000000000000000000000"));
    assert_eq!((w.folder_state(&w.s.f.request), w.folder_state(&w.s.f.zone), w.folder_state(&w.b)), files);
    assert_eq!(w.s.remote_refs(), remote); assert!(!w.s.f.common.join("mortar/locks/folders.lock").exists());
    let mut config = w.s.f.config.clone(); config.zones.remove("b"); w.s.f.configure(&config);
    let r = app(&w, &["sweep"], 0, "swept"); assert_eq!(r.strings("deleted"), &[b]);
    assert_eq!(ref_at(&w, core), Some(w.s.f.base.clone())); assert_eq!(ref_at(&w, b), None);
    let before = observe(&w); let r = app(&w, &["sweep"], 0, "nothing-to-sweep");
    assert!(r.strings("deleted").is_empty()); assert_eq!(observe(&w), before);
}

#[test]
fn build_output_is_untouched_without_idle_days_then_zero_clears_only_mortar_build_dirs() {
    let w = World::new();
    write(&w.s.f.request, "target/deep/out", b"1234567890123"); write(&w.s.f.zone, "target/out", b"1234567");
    write(&w.s.f.git.repo, "target/main", b"main build stays");
    let before = observe(&w); let r = app(&w, &["sweep"], 0, "nothing-to-sweep");
    assert_eq!(r.strings("lines"), &["nothing to sweep", "build dirs not swept: no --idle-days"]);
    assert!(r.is_null("freed_bytes")); assert_eq!(observe(&w), before);
    let old_refs = refs(&w); let old_log = w.s.f.log(); let remote = w.s.remote_refs();
    let r = app(&w, &["sweep", "--idle-days", "0"], 0, "swept");
    let mut removed = r.strings("removed").to_vec(); removed.sort();
    let mut expected = vec![w.s.f.request.join("target").display().to_string(), w.s.f.zone.join("target").display().to_string()]; expected.sort();
    assert_eq!(removed, expected); assert_eq!(r.number("freed_bytes"), 20); assert!(r.strings("deleted").is_empty());
    assert!(!w.s.f.request.join("target").exists()); assert!(!w.s.f.zone.join("target").exists());
    assert_eq!(fs::read(w.s.f.git.repo.join("target/main")).unwrap(), b"main build stays");
    for folder in [&w.s.f.request, &w.s.f.zone, &w.b] {
        assert_eq!(fs::read(folder.join("src/file")).unwrap(), b"base\n");
        assert_eq!(fs::read(folder.join("sub/keep")).unwrap(), b"keep\n");
        assert_eq!(fs::read(folder.join("root.txt")).unwrap(), b"root\n");
    }
    assert_eq!(refs(&w), old_refs); assert_eq!(w.s.f.log(), old_log); assert_eq!(w.s.remote_refs(), remote);
    let before = observe(&w); let r = app(&w, &["sweep", "--idle-days", "0"], 0, "nothing-to-sweep");
    assert_eq!(r.number("freed_bytes"), 0); assert!(r.strings("removed").is_empty()); assert_eq!(observe(&w), before);
}

#[test]
fn refused_extra_ref_deletes_neither_valid_candidates_nor_build_output() {
    let w = World::new();
    for name in ["refs/t3/eligible", "refs/mortar/snap/core/closed"] { raw(&w.s.f.git, &["update-ref", name, w.s.f.base.as_str()]); }
    write(&w.s.f.request, "target/out", b"must remain"); let before = observe(&w);
    let r = app(&w, &["sweep", "--ref", "refs/t3/eligible", "--ref", "refs/heads/main", "--idle-days", "0"], 1, "refused");
    assert_eq!(r.strings("lines"), &["extra ref refs/heads/main is under refs/heads/"]);
    assert_eq!(r.text("next"), "mr sweep with the same words without --ref refs/heads/main, with --actor <name>");
    assert_eq!(observe(&w), before); assert!(!w.s.f.common.join("mortar/locks/folders.lock").exists());
}
