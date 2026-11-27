//! A zone moved during the real join is protected and reported through the CLI.
use crate::{json, support::{actual_git, fixture_partners::Partners, script, shim_path, REQUEST, ZONE}};
use std::{fs, process::{Command, Stdio}};

#[test]
fn zone_moved_sideways_reports_the_protected_join_failure() {
    let f = Partners::new();
    f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    let request = f.tip(REQUEST);
    let tree = f.git.commit_info(&f.base).unwrap().tree;
    let sideways = f.git.commit_tree(&tree, std::slice::from_ref(&f.base), "sideways\n", None, None).unwrap();
    let common = f.git.common_dir().unwrap();
    let dir = common.join("sideways-git");
    fs::create_dir(&dir).unwrap();
    let marker = common.join("sideways-fired");
    let target = common.join("sideways-target");
    script(&dir.join("git"), br##"#!/bin/sh
set -eu
if [ "${3-}" = merge-base ] && [ "${4-}" = --is-ancestor ] && [ "${6-}" = "$RACE_REQUEST" ] && [ "${7-}" = "$RACE_BASE" ] && [ ! -e "$RACE_MARKER" ]; then
  : > "$RACE_MARKER"
  "$REAL_GIT" -C "$2" update-ref refs/heads/zone/core "$RACE_TIP"
fi
if [ "${3-}" = merge-base ] && [ "${5-}" = "$RACE_TIP" ]; then
  printf '%s' "$6" > "$RACE_TARGET"
fi
exec "$REAL_GIT" "$@"
"##);
    let out = Command::new(env!("CARGO_BIN_EXE_mortar"))
        .current_dir(&f.request).env_clear().envs(f.git.env.iter().cloned())
        .env("PATH", shim_path(&dir)).env("HOME", &f.repo.path).env("TMPDIR", &f.repo.path)
        .env("CODETEAM_SESSION", "harness").env("REAL_GIT", actual_git())
        .env("RACE_REQUEST", request.as_str()).env("RACE_BASE", f.base.as_str())
        .env("RACE_TIP", sideways.as_str()).env("RACE_MARKER", &marker).env("RACE_TARGET", &target)
        .stdin(Stdio::null()).args(["--json", "join"]).output().unwrap();
    let r = json::check(out, "join", 5, "failed");
    assert!(marker.exists());
    let proposed = fs::read_to_string(target).unwrap();
    assert_eq!(r.strings("lines"), &[
        "the snapshot ref moved as the work was saved first".to_string(),
        format!("join op log: ref refs/heads/zone/core is protected by join: moving it from {sideways} to {proposed} is not a fast-forward; nothing was changed; move it only to a commit that has {sideways} as an ancestor, or ask join"),
    ]);
    assert!(r.is_null("owner"));
    assert_eq!(r.text("next"), "repeat the same command");
    assert_eq!(f.tip(ZONE), sideways);
    assert_eq!(f.tip(REQUEST), request);
}
