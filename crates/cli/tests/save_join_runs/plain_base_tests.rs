//! Plain joins keep or advance the request boundary, recover a killed rebase, and retry a failed record rewrite.
use crate::support::*;
use std::{fs, os::unix::process::ExitStatusExt};

#[test]
fn plain_base_rebased() {
    let f = Fixture::new(); f.save_topic(); let zone = f.advance_zone(false);
    f.report(&["--json", "join"], "join", 0, "rebased");
    let out = f.app(&["--json", "status"]); let body = String::from_utf8(out.stdout.clone()).unwrap();
    let status = crate::json::read(out, "status", 0);
    assert_eq!(status.text("base"), zone.as_str()); assert!(body.contains("\"partners\":[]"), "{body}");
}

#[test]
fn plain_base_fast_forward() {
    let f = Fixture::new(); f.save_topic();
    f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(f.status().text("base"), f.base.as_str());
}

#[test]
fn plain_base_recovers_killed_rebase() {
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(false);
    let dir = f.common.join("plain-base-shim"); fs::create_dir(&dir).unwrap();
    let marker = f.common.join("plain-base-killed");
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
    let mut cmd = f.command(&f.request, &["--json", "join"]);
    cmd.env("PATH", shim_path(&dir)).env("MORTAR_TEST_FOLDER", &f.request)
        .env("MORTAR_TEST_LOG", oplog::oplog_path(&f.common)).env("MORTAR_TEST_MARKER", &marker)
        .env("MORTAR_TEST_REAL_GIT", actual_git());
    let out = cmd.output().unwrap(); assert_eq!(out.status.signal(), Some(9), "{out:?}");
    assert_eq!(fs::read(marker).unwrap(), b"killed"); assert_ne!(f.head(), old);
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.base, f.base); assert!(record.partners.is_empty());
    f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(f.status().text("base"), zone.as_str());
}

#[test]
fn plain_base_rewrite_failure_finishes_on_retry() {
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(false);
    let temp = f.git.git_dir(&f.request).unwrap().join("mortar-folder.json.tmp");
    fs::create_dir(&temp).unwrap();
    let out = f.app(&["--json", "join"]); fs::remove_dir(&temp).unwrap();
    let new = f.head(); assert_ne!(new, old);
    let report = crate::json::check(out, "join", 5, "failed");
    let prefix = format!("rebased: {REQUEST} moved from {old} to {new}, replayed onto the zone; the folder record was not rewritten and the folder did not move:");
    assert!(report.strings("lines").iter().any(|line| line.starts_with(&prefix)), "{:?}", report.lines());
    // HEAD follows the moved request ref; the index and files show whether the folder moved.
    assert_eq!(f.git.index_tree(&f.request).unwrap(), f.git.commit_info(&old).unwrap().tree);
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"topic work\n");
    assert!(!f.request.join("src/upstream").exists());
    f.report(&["--json", "join"], "join", 0, "joined");
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.base, zone); assert!(record.partners.is_empty());
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"upstream work\n");
}
