//! commit-files in zone and request folders, through the binary and real throwaway repositories.
use std::{fs, path::Path};
use crate::{json::{check, Report}, support::{raw, write, write_hook, Fixture, ZONE}};
use gitio::Git;

#[test]
fn commit_files_hook_refusals_leave_zone_unmoved() {
    let mut failed = Vec::new();
    for hook in ["pre-commit", "commit-msg"] {
        let result = std::panic::catch_unwind(|| {
            let f = Fixture::new(); write(&f.zone, "src/added", b"new\n"); write(&f.zone, "src/file", b"edited\n");
            let before = f.git.read_ref(ZONE).unwrap();
            let zone_git = Git { repo: f.zone.clone(), ..f.git.clone() };
            assert!(raw(&zone_git, &["diff", "--cached", "--name-only"]).is_empty(), "{hook}: initial index");
            write_hook(&f.common, hook, b"#!/bin/sh\necho hook-out-token\necho hook-err-token >&2\nexit 7\n");
            let plain = f.command(&f.zone, &["--actor", "zone-owner", "commit-files", "src/added", "src/file"]).output().unwrap();
            assert_eq!(plain.status.code(), Some(3), "{hook} plain: {plain:?}");
            let text = format!("{}{}", String::from_utf8_lossy(&plain.stdout), String::from_utf8_lossy(&plain.stderr));
            assert!(text.contains("hook-out-token") && text.contains("hook-err-token"), "{hook}: {text}");
            assert_eq!(f.git.read_ref(ZONE).unwrap(), before); assert!(!f.lock_path().exists());
            assert!(raw(&zone_git, &["diff", "--cached", "--name-only"]).is_empty(), "{hook}: refused index");
            let report = commit_files(&f, &f.zone, 3, "refused");
            assert_eq!(report.text("hook"), hook); assert_eq!(report.number("code"), 7);
            assert!(report.text("stdout").contains("hook-out-token")); assert!(report.text("stderr").contains("hook-err-token"));
            assert!(report.text("next").contains(&format!("fix what the {hook} hook reports, then mr commit-files again")));
            assert_eq!(f.git.read_ref(ZONE).unwrap(), before); assert!(!f.lock_path().exists());
            assert!(raw(&zone_git, &["diff", "--cached", "--name-only"]).is_empty(), "{hook}: refused index");
        });
        if result.is_err() { failed.push(hook); }
    }
    assert!(failed.is_empty(), "failed rows: {failed:?}");
}

#[test]
fn commit_files_hook_fixed_after_refusal_commits_named_paths() {
    let f = Fixture::new(); write(&f.zone, "src/added", b"new\n"); write(&f.zone, "src/file", b"edited\n");
    let before = f.git.read_ref(ZONE).unwrap().unwrap();
    let zone_git = Git { repo: f.zone.clone(), ..f.git.clone() };
    assert!(raw(&zone_git, &["diff", "--cached", "--name-only"]).is_empty(), "initial index");
    write_hook(&f.common, "pre-commit", b"#!/bin/sh\necho hook-out-token\necho hook-err-token >&2\nexit 7\n");
    commit_files(&f, &f.zone, 3, "refused");
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(before.clone())); assert!(!f.lock_path().exists());
    assert!(raw(&zone_git, &["diff", "--cached", "--name-only"]).is_empty(), "refused index");
    write_hook(&f.common, "pre-commit", b"#!/bin/sh\nexit 0\n");
    commit_files(&f, &f.zone, 0, "committed");
    let after = f.git.read_ref(ZONE).unwrap().unwrap();
    assert_ne!(after, before);
    assert_eq!(raw(&f.git, &["diff", "--name-status", before.as_str(), after.as_str()]), b"A\tsrc/added\nM\tsrc/file\n");
    assert_eq!(f.blob(&after, "src/added"), b"new\n"); assert_eq!(f.blob(&after, "src/file"), b"edited\n");
    assert!(!f.lock_path().exists());
}

fn commit_files(f: &Fixture, folder: &Path, exit: i32, outcome: &str) -> Report {
    check(f.command(folder, &["--json", "--actor", "zone-owner", "commit-files", "src/added", "src/file"])
        .output().unwrap(), "commit-files", exit, outcome)
}

#[test]
fn commit_files_zone_commits_only_named_paths_and_logs_the_move() {
    let f = Fixture::new();
    let old = f.git.read_ref(ZONE).unwrap().unwrap();
    write(&f.zone, "src/added", b"added\n");
    fs::remove_file(f.zone.join("src/file")).unwrap();
    write(&f.zone, "sub/keep", b"uncommitted edit\n");
    let before = f.records().len();
    let r = commit_files(&f, &f.zone, 0, "committed");
    let tip = f.git.read_ref(ZONE).unwrap().unwrap();
    assert_ne!(tip, old);
    assert_eq!(r.text("commit"), tip.as_str());
    assert_eq!(r.text("ref"), ZONE);
    assert!(r.flag("moved"));
    assert_eq!(f.git.commit_info(&tip).unwrap().parents, vec![old.clone()]);
    assert_eq!(raw(&f.git, &["diff", "--name-status", old.as_str(), tip.as_str()]), b"A\tsrc/added\nD\tsrc/file\n");
    assert_eq!(f.blob(&tip, "src/added"), b"added\n");
    assert_eq!(f.blob(&tip, "sub/keep"), b"keep\n");
    assert_eq!(fs::read(f.zone.join("sub/keep")).unwrap(), b"uncommitted edit\n");
    let zone_git = Git { repo: f.zone.clone(), ..f.git.clone() };
    assert!(String::from_utf8(raw(&zone_git, &["status", "--porcelain"])).unwrap().contains(" M sub/keep"));
    let message = String::from_utf8(raw(&f.git, &["show", "-s", "--format=%B", tip.as_str()])).unwrap();
    assert!(message.contains("src/added") && message.contains("src/file"));
    let records = f.records();
    let moves: Vec<_> = records[before..].iter().filter(|r| r.phase == oplog::OpPhase::Done
        && r.moves.iter().any(|m| m.refname == ZONE)).collect();
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].actor, "zone-owner");
    assert_eq!(moves[0].moves, vec![oplog::RefMove { refname: ZONE.into(), old: Some(old), new: tip }]);
}

#[test]
fn commit_files_zone_repeat_is_same_and_does_not_move_the_ref() {
    let f = Fixture::new();
    write(&f.zone, "src/added", b"added\n");
    fs::remove_file(f.zone.join("src/file")).unwrap();
    commit_files(&f, &f.zone, 0, "committed");
    let tip = f.git.read_ref(ZONE).unwrap().unwrap();
    let r = commit_files(&f, &f.zone, 0, "same");
    assert!(!r.flag("moved"));
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(tip));
}

#[test]
fn commit_files_zone_off_tip_is_refused_with_refresh_next() {
    let f = Fixture::new();
    write(&f.request, "src/joined", b"joined\n");
    check(f.command(&f.request, &["--json", "join"]).output().unwrap(), "join", 0, "joined");
    let tip = f.git.read_ref(ZONE).unwrap().unwrap();
    assert_ne!(tip, f.base);
    let r = commit_files(&f, &f.zone, 3, "refused");
    assert!(r.text("next").contains("mr folder refresh"));
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(tip));
}

#[test]
fn commit_files_request_is_refused_with_save_next_and_no_refs_moved() {
    let f = Fixture::new();
    let before = raw(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let r = commit_files(&f, &f.request, 3, "refused");
    assert!(r.text("next").contains("mr save"));
    assert_eq!(raw(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), before);
}

#[test]
fn commit_files_zone_repeat_after_a_stopped_run_commits_once() {
    let f = Fixture::new();
    let old = f.git.read_ref(ZONE).unwrap().unwrap();
    let zone_git = Git { repo: f.zone.clone(), ..f.git.clone() };
    let git_dir = String::from_utf8(raw(&zone_git, &["rev-parse", "--git-dir"])).unwrap();
    let marker = f.zone.join(git_dir.trim_end()).join("mortar-commit-files-pending");
    write(&f.zone, "src/file", b"stopped edit\n");
    raw(&zone_git, &["add", "src/file"]);
    fs::write(&marker, format!("{old}\nsrc/file\n")).unwrap();
    let before = f.records().len();
    let r = check(f.command(&f.zone, &["--json", "--actor", "zone-owner", "commit-files", "src/file"])
        .output().unwrap(), "commit-files", 0, "committed");
    let tip = f.git.read_ref(ZONE).unwrap().unwrap();
    assert_ne!(tip, old);
    assert_eq!(r.text("commit"), tip.as_str());
    assert_eq!(f.git.commit_info(&tip).unwrap().parents, vec![old.clone()]);
    assert_eq!(raw(&f.git, &["diff", "--name-status", old.as_str(), tip.as_str()]), b"M\tsrc/file\n");
    assert_eq!(f.blob(&tip, "src/file"), b"stopped edit\n");
    assert!(!marker.exists());
    let records = f.records();
    assert_eq!(records[before..].iter().filter(|r| r.phase == oplog::OpPhase::Done
        && r.moves.iter().any(|m| m.refname == ZONE)).count(), 1);
    check(f.command(&f.zone, &["--json", "--actor", "zone-owner", "commit-files", "src/file"])
        .output().unwrap(), "commit-files", 0, "same");
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(tip));
}

#[test]
fn commit_files_zone_repeat_after_a_stopped_run_and_a_revert_names_staged_paths_until_restored() {
    let f = Fixture::new();
    let tip = f.git.read_ref(ZONE).unwrap().unwrap();
    let zone_git = Git { repo: f.zone.clone(), ..f.git.clone() };
    let git_dir = String::from_utf8(raw(&zone_git, &["rev-parse", "--git-dir"])).unwrap();
    let marker = f.zone.join(git_dir.trim_end()).join("mortar-commit-files-pending");
    write(&f.zone, "src/file", b"stopped edit\n");
    raw(&zone_git, &["add", "src/file"]);
    fs::write(&marker, format!("{tip}\nsrc/file\n")).unwrap();
    write(&f.zone, "src/file", b"base\n");
    assert_eq!(raw(&zone_git, &["show", ":src/file"]), b"stopped edit\n");
    for attempt in 1..=2 {
        let output = f.command(&f.zone, &["--json", "--actor", "zone-owner", "commit-files", "src/file"])
            .output().unwrap();
        assert_eq!(output.status.code(), Some(0), "attempt {attempt}: {output:?}");
        let r = check(output, "commit-files", 0, "same");
        assert!(!r.flag("moved"));
        assert_eq!(r.text("next"), "git restore --staged -- src/file");
        assert_eq!(r.strings("staged"), ["src/file"]);
        assert!(r.lines().contains("src/file"));
        assert!(marker.exists());
        assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(tip.clone()));
    }
    raw(&zone_git, &["restore", "--staged", "--", "src/file"]);
    let output = f.command(&f.zone, &["--json", "--actor", "zone-owner", "commit-files", "src/file"])
        .output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains("\"staged\":"));
    let r = check(output, "commit-files", 0, "same");
    assert!(!r.flag("moved"));
    assert_eq!(r.text("next"), "mr squash core --actor <name> carries it in the zone's next squash");
    assert!(!marker.exists());
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(tip));
}

#[test]
fn commit_files_zone_lagging_with_a_staged_named_path_is_refused() {
    let f = Fixture::new();
    write(&f.request, "src/file", b"joined edit\n");
    check(f.command(&f.request, &["--json", "join"]).output().unwrap(), "join", 0, "joined");
    let tip = f.git.read_ref(ZONE).unwrap().unwrap();
    assert_ne!(tip, f.base);
    assert_eq!(f.blob(&tip, "src/file"), b"joined edit\n");
    write(&f.zone, "src/file", b"staged zone edit\n");
    let zone_git = Git { repo: f.zone.clone(), ..f.git.clone() };
    raw(&zone_git, &["add", "src/file"]);
    let r = check(f.command(&f.zone, &["--json", "--actor", "zone-owner", "commit-files", "src/file"])
        .output().unwrap(), "commit-files", 3, "refused");
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(tip.clone()));
    assert_eq!(f.blob(&tip, "src/file"), b"joined edit\n");
    assert!(!r.text("next").contains("mr folder refresh"), "{}", r.text("next"));
}
