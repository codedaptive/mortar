//! CLI run_backup/run_restore_backup and reports joined to land::backup/restore_backup, real Git transport and Oplog.
//! Independent ref/OID inventories prove a round trip, repeat safety, local divergence preservation and hook refusal.
use super::fixture::Harness;
use crate::support::{raw, script, REQUEST, SNAP, ZONE};
use gitio::Oid;
use std::fs;

const SQUASH: &str = "refs/mortar/squash/core";
fn inventory(s: &Harness) -> Vec<(&'static str, Oid)> {
    let request = s.f.commit(&s.f.base, &[("src/request", b"request work\n")], "request\n");
    let zone = s.f.commit(&s.f.base, &[("src/zone", b"zone work\n")], "zone\n");
    let snapshot = s.f.commit(&request, &[("src/snapshot", b"snapshot work\n")], "snapshot\n");
    raw(&s.f.git, &["update-ref", REQUEST, request.as_str()]);
    raw(&s.f.git, &["update-ref", SNAP, snapshot.as_str()]);
    let squash = s.squash(&zone);
    vec![(REQUEST, request), (ZONE, zone), (SNAP, snapshot), (SQUASH, squash.commit)]
}
fn names(entries: &[(&str, Oid)]) -> Vec<String> {
    let mut out: Vec<_> = entries.iter().map(|(name, _)| (*name).to_string()).collect(); out.sort(); out
}
fn sorted(words: &[String]) -> Vec<String> { let mut out = words.to_vec(); out.sort(); out }
fn refs(s: &Harness) -> Vec<u8> { raw(&s.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]) }

#[test]
fn backup_round_trips_all_four_namespaces_into_a_fresh_clone_and_second_backup_pushes_nothing() {
    let s = Harness::new(); let entries = inventory(&s); let expected = names(&entries); let host = oplog::host_name().unwrap();
    let local_before = refs(&s);
    let r = s.app(&["backup"], 0, "backed-up");
    assert_eq!(sorted(r.strings("pushed")), expected); assert!(r.strings("unchanged").is_empty());
    assert_eq!(r.text("next"), "nothing to do"); assert_eq!(refs(&s), local_before);
    let remote = s.git_at(&s.bare);
    for (name, oid) in &entries {
        let backup = format!("refs/mortar-backup/{host}/{}", name.strip_prefix("refs/").unwrap());
        assert_eq!(remote.read_ref(&backup).unwrap(), Some(oid.clone()), "remote backup ref {backup}");
    }
    assert_eq!(remote.read_ref("refs/heads/main").unwrap(), Some(s.f.base.clone()));
    assert_eq!(remote.read_ref("refs/heads/integration").unwrap(), None);
    let before = s.observe(); let r = s.app(&["backup"], 0, "backed-up");
    assert!(r.strings("pushed").is_empty()); assert_eq!(sorted(r.strings("unchanged")), expected);
    assert_eq!(s.observe(), before, "an idle backup must not even append an op");
    assert_eq!(s.f.records().iter().filter(|r| r.kind == "land-backup" && r.phase == oplog::OpPhase::Done).count(), 1);

    let (clone, git) = s.clone_store();
    for (name, _) in &entries { assert_eq!(git.read_ref(name).unwrap(), None, "fresh clone must not already hold {name}"); }
    let remote_before = s.remote_refs();
    let r = s.run(&clone, &["restore-backup", &host], 0, "restored");
    assert_eq!(sorted(r.strings("created")), expected);
    for field in ["same", "differs", "vanished"] { assert!(r.strings(field).is_empty()); }
    for (name, oid) in &entries { assert_eq!(git.read_ref(name).unwrap(), Some(oid.clone()), "restored {name}"); }
    assert_eq!(git.read_ref("refs/heads/main").unwrap(), Some(s.f.base.clone())); assert_eq!(s.remote_refs(), remote_before);
    assert_eq!(raw(&git, &["status", "--porcelain"]), b"");
    let common = git.common_dir().unwrap();
    assert_eq!(oplog::read_op_log(&common).unwrap().records.iter().filter(|r| r.kind == "land-restore-backup" && r.phase == oplog::OpPhase::Done).count(), 1);
    let log = fs::read(oplog::oplog_path(&common)).unwrap();
    let r = s.run(&clone, &["restore-backup", &host], 0, "restored");
    assert!(r.strings("created").is_empty()); assert_eq!(sorted(r.strings("same")), expected);
    assert_eq!(fs::read(oplog::oplog_path(&common)).unwrap(), log); assert_eq!(s.remote_refs(), remote_before);
}

#[test]
fn restore_backup_reports_a_differing_local_ref_without_overwriting_it_and_still_restores_missing_refs() {
    let s = Harness::new(); let entries = inventory(&s); let host = oplog::host_name().unwrap();
    s.app(&["backup"], 0, "backed-up"); let (clone, git) = s.clone_store();
    raw(&git, &["update-ref", ZONE, s.f.base.as_str()]);
    let remote_before = s.remote_refs(); let r = s.run(&clone, &["restore-backup", &host], 0, "restored");
    assert_eq!(r.strings("differs"), &[ZONE]);
    let created: Vec<_> = entries.iter().filter(|(name, _)| *name != ZONE).cloned().collect();
    assert_eq!(sorted(r.strings("created")), names(&created)); assert!(r.strings("same").is_empty()); assert!(r.strings("vanished").is_empty());
    let zone = &entries.iter().find(|(name, _)| *name == ZONE).unwrap().1;
    assert!(r.strings("lines").contains(&format!("{ZONE} is at {}, its backup at {zone}: left as it is", s.f.base)));
    assert_eq!(r.text("next"), format!("each ref left as it is holds another commit than its backup under refs/mortar-backup/{host}/ on the remote; keep it, or move it to the backup's commit by hand"));
    assert_eq!(git.read_ref(ZONE).unwrap(), Some(s.f.base.clone()));
    for (name, oid) in created { assert_eq!(git.read_ref(name).unwrap(), Some(oid)); }
    assert_eq!(s.remote_refs(), remote_before);
}

#[test]
fn restoring_a_host_with_no_backup_is_an_inert_restored_report_not_a_refusal() {
    let s = Harness::new(); let before = s.observe();
    let r = s.app(&["restore-backup", "bisect-backup-no-backup"], 0, "restored");
    assert_eq!(r.strings("lines"), &["host bisect-backup-no-backup has no backup on the remote"]);
    for field in ["created", "same", "differs", "vanished"] { assert!(r.strings(field).is_empty()); }
    assert_eq!(r.text("next"), "nothing to do"); assert_eq!(s.observe(), before);
}

#[test]
fn a_real_remote_hook_refuses_backup_with_cli_words_and_no_remote_or_local_ref_movement() {
    let s = Harness::new(); inventory(&s);
    script(&s.bare.join("hooks/pre-receive"), b"#!/bin/sh\nwhile read old new ref; do\ncase \"$ref\" in refs/mortar-backup/*) printf 'backup rejected\\n' >\"$(git rev-parse --git-dir)/bisect-backup-hook-ran\"; exit 1;; esac\ndone\n");
    let remote_before = s.remote_refs(); let local_before = refs(&s);
    let r = s.app(&["backup"], 1, "refused");
    assert_eq!(r.text("owner"), "the remote origin");
    assert_eq!(r.text("next"), "clear what the remote origin refuses for refs/mortar-backup/, then repeat the same command");
    assert_eq!(r.strings("lines").len(), 1);
    assert!(r.strings("lines")[0].starts_with("the remote origin refused to move refs/mortar-backup/"));
    assert!(r.strings("lines")[0].ends_with(": a hook on the remote declined it"));
    assert_eq!(fs::read(s.bare.join("bisect-backup-hook-ran")).unwrap(), b"backup rejected\n");
    assert_eq!(s.remote_refs(), remote_before); assert_eq!(refs(&s), local_before);
    assert!(!s.f.common.join("mortar/locks/land.lock").exists());
}
