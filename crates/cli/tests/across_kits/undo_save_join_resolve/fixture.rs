//! Reuses after_publish's and snapshot_overlap_disk_sweep's owned repositories and full folder/ref/log observation,
//! not a second whole-kit fake.
pub use crate::snapshot_overlap_disk_sweep::fixture::{observe, ref_at, World};
use crate::{json::{check, Report}, support::{raw, actual_git, script, shim_path, REQUEST}};
use gitio::Oid;
use oplog::{OpPhase, OpRecord};
use std::{fs, os::unix::process::ExitStatusExt, path::Path};

pub fn agent(w: &World, path: &Path, words: &[&str], exit: i32, outcome: &str) -> Report {
    let mut c = w.s.f.command(path, &["--json"]); c.args(words);
    check(c.output().unwrap(), words[0], exit, outcome)
}
pub fn ours(w: &World, words: &[&str], exit: i32, outcome: &str) -> Report { agent(w, &w.s.f.request, words, exit, outcome) }
pub fn done(w: &World, kind: &str) -> Vec<OpRecord> {
    w.s.f.records().into_iter().filter(|r| r.kind == kind && r.phase == OpPhase::Done).collect()
}
pub fn rebase(w: &World) -> (Oid, Oid, Oid) {
    let old = w.s.f.save_topic(); let zone = w.s.f.advance_zone(false);
    ours(w, &["join"], 0, "rebased"); let new = w.s.f.head();
    assert_ne!(new, old); assert_eq!(w.s.f.git.commit_info(&new).unwrap().parents, vec![zone.clone()]);
    assert_eq!(fs::read(w.s.f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    (old, new, zone)
}
pub fn index_at(w: &World, path: &Path, commit: &Oid) {
    assert_eq!(w.s.f.git.index_tree(path).unwrap(), w.s.f.git.commit_info(commit).unwrap().tree);
}
pub fn assert_one_undo(w: &World, target: &OpRecord, old: &Oid, new: &Oid) -> OpRecord {
    let records = done(w, "undo"); assert_eq!(records.len(), 1);
    let r = records[0].clone(); assert_eq!(r.actor, "core/task");
    assert_eq!(r.key, format!("undo:{}", target.op_id));
    assert_eq!(r.reason, format!("undo of {} op {}", target.kind, target.op_id));
    assert_eq!(r.moves.len(), 1); assert_eq!(r.moves[0].refname, REQUEST);
    assert_eq!(r.moves[0].old, Some(old.clone())); assert_eq!(r.moves[0].new, *new); r
}
/// Kills the actual CLI at its real index/folder boundary, only after the chosen operation has reached Git.
/// All earlier Git calls execute normally; no clock, synthetic answer or live OS service is the oracle.
pub fn kill_before_step(w: &World, verb: &str, kind: &str, flag: &str) {
    let dir = w.s.f.common.join(format!("undo-save-join-resolve-{verb}-shim")); fs::create_dir(&dir).unwrap();
    let marker = w.s.f.common.join(format!("undo-save-join-resolve-{verb}-killed"));
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${2-}" = "$MORTAR_VALIDATION_FOLDER" ] && [ "${3-}" = read-tree ] && [ "${4-}" = "$MORTAR_VALIDATION_FLAG" ] &&
   grep -q "\"kind\":\"$MORTAR_VALIDATION_KIND\"" "$MORTAR_VALIDATION_LOG"; then
  printf 'killed before real folder step\n' >"$MORTAR_VALIDATION_MARKER"
  kill -KILL "$PPID"
  exit 97
fi
exec "$MORTAR_VALIDATION_REAL_GIT" "$@"
"#);
    let mut c = w.s.f.command(&w.s.f.request, &["--json", verb]);
    c.env("PATH", shim_path(&dir)).env("MORTAR_VALIDATION_FOLDER", &w.s.f.request).env("MORTAR_VALIDATION_FLAG", flag)
        .env("MORTAR_VALIDATION_KIND", kind).env("MORTAR_VALIDATION_LOG", oplog::oplog_path(&w.s.f.common))
        .env("MORTAR_VALIDATION_MARKER", &marker).env("MORTAR_VALIDATION_REAL_GIT", actual_git());
    let child = c.spawn().unwrap(); let pid = child.id(); let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.signal(), Some(9), "{output:?}");
    assert_eq!(fs::read(marker).unwrap(), b"killed before real folder step\n");
    assert!(!oplog::PidProbe::pid_alive(&oplog::KillPidProbe, pid), "the killed lock holder must be reaped");
    assert_eq!(done(w, kind).len(), 1, "the real op must be durably Done before the missing folder step");
}
pub fn clean_index_bytes(w: &World) -> Vec<u8> { raw(&w.s.f.local(), &["ls-files", "--stage", "-z"]) }
