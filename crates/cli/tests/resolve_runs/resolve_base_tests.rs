//! A plain resolve records the zone tip; a killed or failed record rewrite is finished by the next save.
use crate::{paused::Paused, support::{actual_git, raw, script, shim_path, REQUEST}};
use std::{fs, os::unix::process::ExitStatusExt};

#[test]
fn resolve_base_plain_rebase() {
    let r = Paused::new(false); r.first();
    r.f.write("src/file", b"chosen resolution\n");
    r.f.report(&["--json", "resolve"], "resolve", 0, "rebased");
    assert_eq!(r.f.status().text("base"), r.zone.as_str());
}

#[test]
fn resolve_base_recovers_kill_before_record_rewrite() {
    let r = Paused::new(false); r.first(); let f = &r.f;
    f.write("src/file", b"chosen resolution\n");
    let dir = f.common.join("resolve-base-shim"); fs::create_dir(&dir).unwrap();
    let marker = f.common.join("resolve-base-killed");
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${2-}" = "$MORTAR_TEST_FOLDER" ] && [ "${3-}" = rev-parse ] &&
   [ "${4-}" = --path-format=absolute ] && [ "${5-}" = --git-dir ] &&
   grep -q '"kind":"join-rebase"' "$MORTAR_TEST_LOG"; then
  printf killed >"$MORTAR_TEST_MARKER"
  kill -KILL "$PPID"
  exit 97
fi
exec "$MORTAR_TEST_REAL_GIT" "$@"
"#);
    let mut cmd = f.command(&f.request, &["--json", "resolve"]);
    cmd.env("PATH", shim_path(&dir)).env("MORTAR_TEST_FOLDER", &f.request)
        .env("MORTAR_TEST_LOG", oplog::oplog_path(&f.common)).env("MORTAR_TEST_MARKER", &marker)
        .env("MORTAR_TEST_REAL_GIT", actual_git());
    let out = cmd.output().unwrap(); assert_eq!(out.status.signal(), Some(9), "{out:?}");
    assert_eq!(fs::read(marker).unwrap(), b"killed");
    assert_eq!(f.last_move().record.kind, "join-rebase");
    assert_eq!(f.git.commit_info(&f.head()).unwrap().parents, vec![r.zone.clone()]);
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.base, f.base); assert!(record.partners.is_empty());

    crate::json::read(f.app(&["--json", "save", "Finish the killed resolve."]), "save", 0);
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.base, r.zone); assert!(record.partners.is_empty());
}

#[test]
fn resolve_base_rewrite_failure_finishes_on_save() {
    let r = Paused::new(false); r.first(); let f = &r.f;
    f.write("src/file", b"chosen resolution\n");
    let index = raw(&f.local(), &["ls-files", "--stage", "-z"]);
    let temp = f.git.git_dir(&f.request).unwrap().join("mortar-folder.json.tmp");
    fs::create_dir(&temp).unwrap();
    let out = f.app(&["--json", "resolve"]); fs::remove_dir(&temp).unwrap();
    let new = f.head(); assert_ne!(new, r.old);
    let report = crate::json::check(out, "resolve", 5, "failed");
    let prefix = format!("rebased: {REQUEST} moved from {} to {new}; the folder record was not rewritten and the folder did not move:", r.old);
    assert!(report.strings("lines").iter().any(|line| line.starts_with(&prefix)), "{:?}", report.lines());
    // HEAD follows the moved request ref; the index and files show whether the folder moved.
    assert_eq!(raw(&f.local(), &["ls-files", "--stage", "-z"]), index);
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"chosen resolution\n");
    assert!(!f.request.join("src/upstream").exists());
    crate::json::read(f.app(&["--json", "save", "Finish the failed resolve."]), "save", 0);
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.base, r.zone); assert!(record.partners.is_empty());
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
}
