//! Kills the actual CLI child at the Git boundary AFTER the integration op and BEFORE its folder read-tree.
//! No sleeps, elapsed-time oracle or fake land: all Git calls except the selected boundary run the real program.
//! Joins run_land/run_back_out/run_publish, finish_stopped_move, Land operations and Oplog stale-lock recovery.
use super::fixture::Landing;
use crate::support::{actual_git, script, shim_path};
use oplog::OpPhase;
use std::{fs, os::unix::process::ExitStatusExt};

#[test]
fn a_killed_landing_back_out_or_origin_moved_publish_is_finished_once_by_the_plain_repeat() {
    for verb in ["land", "back-out", "publish"] {
        let s = Landing::new(true);
        let (old, tip, kind) = if verb == "land" { (s.f.base.clone(), s.squash.commit.clone(), "land") }
        else {
            s.land();
            let tip = if verb == "publish" { s.remote_move() } else { s.f.base.clone() };
            (s.squash.commit.clone(), tip, "land-back-out")
        };
        let before_index = s.index(); let before_bytes = s.bytes();
        let shim = s.f.common.join("land-flow-kill-shim"); fs::create_dir(&shim).unwrap();
        let marker = s.f.common.join("land-flow-killed-before-folder");
        script(&shim.join("git"), br#"#!/bin/sh
set -eu
if [ "${2-}" = "$MORTAR_VALIDATION_FOLDER" ] && [ "${3-}" = read-tree ] && [ "${4-}" = --no-sparse-checkout ]; then
  printf 'killed before folder read-tree\n' >"$MORTAR_VALIDATION_MARKER"
  kill -KILL "$PPID"
  exit 97
fi
exec "$MORTAR_VALIDATION_REAL_GIT" "$@"
"#);
        let word = (verb == "land").then_some(s.squash.commit.as_str());
        let output = s.command(verb, word).env("PATH", shim_path(&shim)).env("MORTAR_VALIDATION_FOLDER", &s.folder)
            .env("MORTAR_VALIDATION_MARKER", &marker).env("MORTAR_VALIDATION_REAL_GIT", actual_git()).output().unwrap();
        assert_eq!(output.status.signal(), Some(9), "{verb}: {output:?}");
        assert_eq!(fs::read(&marker).unwrap(), b"killed before folder read-tree\n");
        assert_eq!(s.tip(), tip); assert_eq!(s.index(), before_index); assert_eq!(s.bytes(), before_bytes);
        let records = s.f.records();
        let moves: Vec<_> = records.iter().filter(|r| r.kind == kind && r.phase == OpPhase::Done).collect();
        assert_eq!(moves.len(), 1); assert_eq!(moves[0].moves.len(), 1);
        assert_eq!(moves[0].moves[0].old, Some(old.clone())); assert_eq!(moves[0].moves[0].new, tip);
        let outcome = match verb { "land" => "landed", "back-out" => "nothing-to-back-out", "publish" => "nothing-to-publish", _ => unreachable!() };
        let r = s.run(verb, word, 0, outcome); s.assert_finished(&r, &old, &tip);
        s.assert_clean_at(&tip, if verb == "land" { b"zone work\n" } else if verb == "back-out" { b"base\n" } else { b"remote work\n" });
        assert!(!s.f.common.join("mortar/locks/land.lock").exists());
        assert_eq!(s.f.records().iter().filter(|r| r.kind == kind && r.phase == OpPhase::Done).count(), 1);
        let before = s.observe(); s.run(verb, word, 0, outcome); assert_eq!(s.observe(), before);
    }
}
