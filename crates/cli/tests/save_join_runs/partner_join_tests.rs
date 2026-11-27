use crate::{json, support::{fixture_partners::{partners, text, Partners}, actual_git, raw, script, shim_path, REQUEST, ZONE}};
use std::{fs, os::unix::process::ExitStatusExt, process::{Command, Stdio}};

#[test]
fn partner_join_unmoved_replays_only_request_and_repeats() {
    let f = Partners::new(); f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    json::check(f.app("join"), "join", 0, "joined");
    let tip = f.zone();
    assert_eq!(f.git.commit_info(&tip).unwrap().parents, vec![f.base.clone()]);
    assert_eq!(raw(&f.git, &["show", &format!("{tip}:web/w.txt")]), b"web base\n");
    assert_eq!(f.tip(REQUEST), tip);
    assert_eq!(f.local().rev_parse("HEAD").unwrap(), Some(tip.clone()));
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty());
    let out = f.app("status"); let body = text(&out);
    let status = json::read(out, "status", 0);
    assert!(status.flag("saved")); assert!(status.flag("joined"));
    assert_eq!(status.text("base"), f.base.as_str());
    assert!(body.contains("\"partners\":[]"));
    json::check(f.app("join"), "join", 0, "joined");
    assert_eq!(f.zone(), tip); assert_eq!(f.tip(REQUEST), tip);
}

#[test]
fn partner_join_killed_before_folder_move_finishes_on_repeat() {
    let f = Partners::new(); f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    let old = f.tip(REQUEST);
    let common = f.git.common_dir().unwrap();
    let dir = common.join("partner-join-git-shim"); fs::create_dir(&dir).unwrap();
    let marker = common.join("partner-join-killed");
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${2-}" = "$MORTAR_TEST_FOLDER" ] && [ "${3-}" = read-tree ] && [ "${4-}" = --no-sparse-checkout ] &&
   grep -q '"kind":"join-zone-request"' "$MORTAR_TEST_LOG"; then
  printf killed >"$MORTAR_TEST_MARKER"
  kill -KILL "$PPID"
  exit 97
fi
exec "$MORTAR_TEST_REAL_GIT" "$@"
"#);
    let out = Command::new(env!("CARGO_BIN_EXE_mortar"))
        .current_dir(&f.request).env_clear().envs(f.git.env.iter().cloned())
        .env("PATH", shim_path(&dir)).env("HOME", &f.repo.path).env("TMPDIR", &f.repo.path)
        .env("CODETEAM_SESSION", "harness").env("MORTAR_TEST_FOLDER", &f.request)
        .env("MORTAR_TEST_LOG", oplog::oplog_path(&common)).env("MORTAR_TEST_MARKER", &marker)
        .env("MORTAR_TEST_REAL_GIT", actual_git()).stdin(Stdio::null()).args(["--json", "join"]).output().unwrap();
    assert_eq!(out.status.signal(), Some(9), "{out:?}");
    assert_eq!(fs::read(marker).unwrap(), b"killed");
    let tip = f.zone();
    assert_ne!(tip, old); assert_eq!(f.tip(REQUEST), tip);
    assert_eq!(f.git.index_tree(&f.request).unwrap(), f.git.commit_info(&old).unwrap().tree);
    let report = json::check(f.app("join"), "join", 0, "joined");
    assert!(!report.is_null("finished_move_from"));
    assert_eq!(report.text("finished_move_from"), old.as_str());
    assert_eq!(report.text("finished_move_to"), tip.as_str());
    assert_eq!(f.tip(REQUEST), tip); assert_eq!(f.zone(), tip);
    assert_eq!(f.local().rev_parse("HEAD").unwrap(), Some(tip.clone()));
    assert_eq!(f.git.index_tree(&f.request).unwrap(), f.git.commit_info(&tip).unwrap().tree);
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty());
    assert_eq!(raw(&f.git, &["show", &format!("{tip}:web/w.txt")]), b"web base\n");
    let status = json::read(f.app("status"), "status", 0);
    assert!(status.flag("saved")); assert!(status.flag("joined"));
}

#[test]
fn partner_join_killed_before_record_rewrite_finishes_on_repeat() {
    let f = Partners::new(); f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    let old = f.tip(REQUEST);
    let common = f.git.common_dir().unwrap();
    let dir = common.join("partner-rewrite-git-shim"); fs::create_dir(&dir).unwrap();
    let marker = common.join("partner-rewrite-killed");
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${2-}" = "$MORTAR_TEST_FOLDER" ] && [ "${3-}" = rev-parse ] &&
   [ "${4-}" = --path-format=absolute ] && [ "${5-}" = --git-dir ] &&
   grep -q '"kind":"join-zone-request"' "$MORTAR_TEST_LOG"; then
  printf killed >"$MORTAR_TEST_MARKER"
  kill -KILL "$PPID"
  exit 97
fi
exec "$MORTAR_TEST_REAL_GIT" "$@"
"#);
    let out = Command::new(env!("CARGO_BIN_EXE_mortar"))
        .current_dir(&f.request).env_clear().envs(f.git.env.iter().cloned())
        .env("PATH", shim_path(&dir)).env("HOME", &f.repo.path).env("TMPDIR", &f.repo.path)
        .env("CODETEAM_SESSION", "harness").env("MORTAR_TEST_FOLDER", &f.request)
        .env("MORTAR_TEST_LOG", oplog::oplog_path(&common)).env("MORTAR_TEST_MARKER", &marker)
        .env("MORTAR_TEST_REAL_GIT", actual_git()).stdin(Stdio::null()).args(["--json", "join"]).output().unwrap();
    assert_eq!(out.status.signal(), Some(9), "{out:?}");
    assert_eq!(fs::read(marker).unwrap(), b"killed");
    assert_ne!(f.zone(), old); assert_eq!(f.tip(REQUEST), f.zone());
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.partners.len(), 1);
    json::check(f.app("join"), "join", 0, "joined");
    let out = f.app("status"); let body = text(&out);
    let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), f.base.as_str());
    assert!(body.contains("\"partners\":[]"), "{body}");
    assert!(!status.flag("partner_moved"));
}

#[test]
fn partner_base_recovers_killed_rebase() {
    let f = Partners::new(); f.set();
    let web = f.advance("refs/heads/zone/web", &[("web/w.txt", b"web moved\n")]);
    f.commit_request("src/core/c.txt", b"request work\n");
    let common = f.git.common_dir().unwrap();
    let dir = common.join("partner-rebase-shim"); fs::create_dir(&dir).unwrap();
    let marker = common.join("partner-rebase-killed");
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
    let out = Command::new(env!("CARGO_BIN_EXE_mortar"))
        .current_dir(&f.request).env_clear().envs(f.git.env.iter().cloned())
        .env("PATH", shim_path(&dir)).env("HOME", &f.repo.path).env("TMPDIR", &f.repo.path)
        .env("CODETEAM_SESSION", "harness").env("MORTAR_TEST_FOLDER", &f.request)
        .env("MORTAR_TEST_LOG", oplog::oplog_path(&common)).env("MORTAR_TEST_MARKER", &marker)
        .env("MORTAR_TEST_REAL_GIT", actual_git()).stdin(Stdio::null()).args(["--json", "join"]).output().unwrap();
    assert_eq!(out.status.signal(), Some(9), "{out:?}"); assert_eq!(fs::read(marker).unwrap(), b"killed");
    let tip = f.tip(REQUEST); let parents = f.git.commit_info(&tip).unwrap().parents;
    assert_eq!(parents.len(), 1); let merge = &parents[0];
    assert_eq!(f.git.commit_info(merge).unwrap().parents, vec![f.base.clone(), web.clone()]);
    // mr save runs the pending-move finisher and joins nothing, so the record it leaves is the finisher's alone.
    json::read(f.command(&f.request, &["--json", "save", "Finish the killed join."]), "save", 0);
    let out = f.app("status"); let body = text(&out); let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), merge.as_str()); assert!(body.contains(&partners(&web)), "{body}");
    // Zone and partner are where the new merge has them, so the join is ZoneJoined: base the zone's tip before it.
    json::check(f.app("join"), "join", 0, "joined");
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!((record.base, record.partners.len()), (f.base.clone(), 0));
}

#[test]
fn partner_join_remerge_drops_partners_already_in_zone() {
    let f = Partners::new(); f.set();
    raw(&f.git, &["update-ref", ZONE, f.web.as_str()]);
    let zone = f.advance(ZONE, &[]);
    f.commit_request("src/core/c.txt", b"request work\n");
    let out = f.app("join"); let body = text(&out);
    let report = json::check(out, "join", 0, "rebased");
    assert_eq!(report.text("next"), "mr join");
    assert!(!body.contains("\"partners\":"), "{body}");
    let out = f.app("status"); let body = text(&out);
    let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), zone.as_str());
    assert!(body.contains("\"partners\":[]"), "{body}");
}

#[test]
fn partner_join_moved_remerges_and_rewrites_record() {
    let f = Partners::new(); f.set();
    let web = f.advance("refs/heads/zone/web", &[("web/w.txt", b"web moved\n")]);
    f.commit_request("src/core/c.txt", b"request work\n");
    let report = json::check(f.app("join"), "join", 0, "rebased");
    assert_eq!(report.text("next"), "run the unit tests, then mr join");
    assert_eq!(f.zone(), f.base);
    let tip = f.tip(REQUEST);
    assert_eq!(f.local().rev_parse("HEAD").unwrap(), Some(tip.clone()));
    let parents = f.git.commit_info(&tip).unwrap().parents;
    assert_eq!(parents.len(), 1);
    let merge = &parents[0];
    assert_eq!(f.git.commit_info(merge).unwrap().parents, vec![f.base.clone(), web.clone()]);
    let out = f.app("status"); let body = text(&out);
    let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), merge.as_str());
    assert!(body.contains(&partners(&web)), "{body}");
}

#[test]
fn partner_join_undo_restores_cut_boundary_and_rejoin() {
    let f = Partners::new(); let cut_merge = f.set();
    let web = f.advance("refs/heads/zone/web", &[("web/w.txt", b"web moved\n")]);
    f.commit_request("src/core/c.txt", b"request work\n");
    json::check(f.app("join"), "join", 0, "rebased");
    let merge = f.git.commit_info(&f.tip(REQUEST)).unwrap().parents[0].clone();
    assert_eq!(f.git.commit_info(&merge).unwrap().parents, vec![f.base.clone(), web.clone()]);
    let out = f.app("status"); let body = text(&out); let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), merge.as_str()); assert!(body.contains(&partners(&web)), "{body}");

    json::check(f.app("undo"), "undo", 0, "undone");
    let out = f.app("status"); let body = text(&out); let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), cut_merge.as_str()); assert!(body.contains(&partners(&f.web)), "{body}");

    let report = json::check(f.app("join"), "join", 0, "rebased");
    let remerge = f.git.commit_info(&f.tip(REQUEST)).unwrap().parents[0].clone();
    assert_eq!(f.git.commit_info(&remerge).unwrap().parents, vec![f.base.clone(), web.clone()]);
    assert_eq!(report.text("base"), remerge.as_str());
    let out = f.app("status"); let body = text(&out); let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), report.text("base")); assert!(body.contains(&partners(&web)), "{body}");
}

#[test]
fn partner_join_conflicting_remerge_refuses_without_movement() {
    let f = Partners::new(); f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    f.advance("refs/heads/zone/web", &[("Cargo.toml", b"web\n")]);
    let zone = f.advance(ZONE, &[("Cargo.toml", b"core\n")]);
    let request = f.tip(REQUEST);
    let report = json::check(f.app("join"), "join", 3, "refused");
    assert_eq!(report.text("partner"), "web");
    assert!(report.strings("files").contains(&"Cargo.toml".to_string()));
    assert_eq!(f.zone(), zone); assert_eq!(f.tip(REQUEST), request);
    assert!(!f.git.git_dir(&f.request).unwrap().join("mortar-join.json").exists());
}

#[test]
fn partner_resolve_leaves_set_after_replay_conflict() {
    let f = Partners::new();
    f.advance("refs/heads/zone/web", &[("Cargo.toml", b"web\n")]);
    f.set();
    f.commit_request("Cargo.toml", b"request\n");
    json::check(f.app("join"), "join", 1, "conflicted");
    assert_eq!(f.app("resolve").status.code(), Some(1));
    fs::write(f.request.join("Cargo.toml"), b"request\n").unwrap();
    let report = json::check(f.app("resolve"), "resolve", 0, "rebased");
    assert_eq!(report.strings("left_partners"), &["web"]);
    assert_eq!(report.text("next"), "mr join");
    let out = f.app("status"); let body = text(&out);
    let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), f.base.as_str());
    assert!(body.contains("\"partners\":[]"));
}
