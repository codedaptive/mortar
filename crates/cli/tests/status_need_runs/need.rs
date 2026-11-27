// The wrong-argument-count usage case is in run_need_tests.rs.
use crate::{json::check, need_fixture::Need, observe::*, support::*};
use oplog::{with_lock, KillPidProbe, LockName};
use std::{fs, path::PathBuf};

#[test]
fn need_expands_real_cone_records_gap_and_repeats_once_with_status_next() {
    // mortar need -> join lock/recover -> folders::need -> actual sparse cone/record -> read_status next.
    let n = Need::new(); let refs = raw(&n.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let first = n.report(&["--json", "need", "sub/"], 0, "needed");
    assert_eq!(first.text("path"), "sub"); assert!(first.flag("gap"));
    assert_eq!(n.paths(), vec!["sub"]); assert_eq!(fs::read(n.folder.join("sub/keep")).unwrap(), b"keep\n");
    assert_eq!(n.f.git.sparse_list(&n.folder).unwrap(), vec![PathBuf::from("src"), PathBuf::from("sub")]);
    // A fresh folder: its branch at its base and its tree the tip's, so it reads saved with no work yet.
    let current = status_at(&n.f, &n.folder, "saved");
    assert_eq!(current.strings("gaps"), &["sub"]); assert_eq!(first.text("next"), current.text("next"));
    assert_eq!(raw(&n.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), refs);
    let old = before(&n.f, &n.folder);
    let repeat = n.report(&["--json", "need", "sub"], 0, "already-done");
    assert!(repeat.flag("gap")); assert_eq!(repeat.text("next"), current.text("next"));
    assert_eq!(n.paths(), vec!["sub"]); assert_eq!(before(&n.f, &n.folder), old);
    assert!(!n.f.lock_path().exists());
}

#[test]
fn need_bad_or_absent_paths_refuse_before_record_cone_or_refs_change() {
    // read_need_path -> real Folders plan and Gitio path_kind; no lock, record or sparse mutation.
    for path in ["../x", "/absolute", "not-present"] {
        let n = Need::new(); let old = before(&n.f, &n.folder);
        let cone = n.f.git.sparse_list(&n.folder).unwrap();
        let report = n.report(&["--json", "need", path], 3, "refused");
        if path == "not-present" {
            assert!(report.lines().contains(&format!("not-present is not in the repository at the folder's commit {}", n.f.base)));
            assert_eq!(report.text("commit"), n.f.base.as_str()); assert_eq!(report.text("path"), path);
        }
        assert_eq!(before(&n.f, &n.folder), old); assert_eq!(n.f.git.sparse_list(&n.folder).unwrap(), cone);
        assert!(!n.folder.join("sub/keep").exists()); assert!(!n.f.lock_path().exists());
    }
}

#[test]
fn need_live_join_and_folders_locks_refuse_without_changing_the_record() {
    // Real live parent lock -> child need; both outer join and inner folders lock refusal paths.
    for name in [LockName::Join("core".into()), LockName::Folders] {
        let n = Need::new(); oplog::recover(&n.f.git, &n.f.common).unwrap();
        let old = before(&n.f, &n.folder);
        let output = with_lock(&n.f.git, &n.f.common, &name, "held-for-need-validation", &KillPidProbe,
            || Ok::<_, std::io::Error>(n.f.command(&n.folder, &["--json", "need", "sub"]).output().unwrap())).unwrap();
        let report = check(output, "need", 4, "busy");
        assert!(report.lines().contains("held-for-need-validation"));
        assert!(report.lines().contains(&std::process::id().to_string()));
        assert_eq!(before(&n.f, &n.folder), old); assert!(!n.folder.join("sub/keep").exists());
        assert!(!n.f.lock_path().exists());
    }
}

#[test]
fn need_missing_zone_ref_refuses_without_expanding_the_cone_or_record() {
    // read_branch_tips header: missing zone is a need refusal with nothing changed, not a partial successful need.
    let n = Need::new(); raw(&n.f.git, &["update-ref", "-d", ZONE]);
    let old = before(&n.f, &n.folder); let cone = n.f.git.sparse_list(&n.folder).unwrap();
    let report = n.report(&["--json", "need", "sub"], 3, "refused");
    assert_eq!(report.text("owner"), "the zone's coordinator");
    assert_eq!(report.lines(), format!("the branch {ZONE} does not exist"));
    assert_eq!(before(&n.f, &n.folder), old, "a missing prerequisite must not silently complete need");
    assert_eq!(n.f.git.sparse_list(&n.folder).unwrap(), cone); assert!(!n.folder.join("sub/keep").exists());
}

#[test]
fn need_with_a_request_name_no_branch_can_carry_fails_in_the_clis_words() {
    let n = Need::new();
    let record = record_path(&n.f, &n.folder);
    let original = fs::read_to_string(&record).unwrap();
    assert_eq!(original.matches("\"request\":\"need\"").count(), 1);
    let changed = original.replace("\"request\":\"need\"", "\"request\":\"has space\"");
    assert_eq!(folders::record::decode_folder_record(&changed).unwrap().request.as_deref(), Some("has space"));
    fs::write(&record, changed).unwrap();
    let old = before(&n.f, &n.folder);
    let cone = n.f.git.sparse_list(&n.folder).unwrap();

    let report = n.report(&["--json", "need", "sub"], 5, "failed");
    assert!(report.strings("lines").contains(&"request has space in zone core is not a name a branch can carry".to_string()));
    assert!(report.strings("lines").iter().all(|line| !line.contains("request name \"has space\"")));
    assert_eq!(before(&n.f, &n.folder), old);
    assert_eq!(n.f.git.sparse_list(&n.folder).unwrap(), cone);
    assert!(!n.folder.join("sub/keep").exists());
    assert!(!n.f.lock_path().exists());
}

#[test]
fn need_unborn_head_is_refused_without_record_or_cone_changes() {
    // Public request folder whose symbolic HEAD no longer resolves; read_need_path's HEAD refusal.
    let n = Need::new(); raw(&n.f.git, &["update-ref", "-d", "refs/heads/req/core/need"]);
    let old = before(&n.f, &n.folder); let cone = n.f.git.sparse_list(&n.folder).unwrap();
    let report = n.report(&["--json", "need", "sub"], 3, "refused");
    assert_eq!(report.text("owner"), "the folder's HEAD");
    assert_eq!(report.text("next"), "check out the request's branch in the folder, then repeat the same command");
    assert_eq!(report.strings("lines"), &[format!("HEAD of the folder {} has no commit", n.folder.display())]);
    assert_eq!(before(&n.f, &n.folder), old); assert_eq!(n.f.git.sparse_list(&n.folder).unwrap(), cone);
    assert!(!n.folder.join("sub/keep").exists()); assert!(!n.f.lock_path().exists());
}

#[test]
fn need_attempt_folder_is_refused_without_mutation() {
    let f = Fixture::new(); let attempts = f.root.join("attempts");
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_attempts(&setup(&f.git, &attempts), "core", "task", &["src".into()], 1).unwrap();
    let folder = attempts.join("try-1"); let old = before(&f, &folder);
    check(f.command(&folder, &["--json", "need", "sub"]).output().unwrap(), "need", 3, "refused");
    assert_eq!(before(&f, &folder), old);
}

#[test]
fn need_sparse_add_failure_reports_failure_and_repeat_finishes_the_recorded_path() {
    // Folders writes its durable needed_paths first: a failed sparse-add must not lose it; repeat completes it.
    let n = Need::new(); let dir = n.f.common.join("need-git-shim"); fs::create_dir(&dir).unwrap();
    script(&dir.join("git"), b"#!/bin/sh\nset -eu\nif [ \"${3-}\" = sparse-checkout ] && [ \"${4-}\" = add ]; then\n  printf 'fixture sparse add refused\\n' >&2; exit 73\nfi\nexec \"$MORTAR_VALIDATION_REAL_GIT\" \"$@\"\n");
    let output = n.f.command(&n.folder, &["--json", "need", "sub"])
        .env("PATH", shim_path(&dir)).env("MORTAR_VALIDATION_REAL_GIT", actual_git()).output().unwrap();
    let report = check(output, "need", 5, "failed"); assert!(report.lines().contains("fixture sparse add refused"));
    assert_eq!(n.paths(), vec!["sub"]); assert!(!n.folder.join("sub/keep").exists()); assert!(!n.f.lock_path().exists());
    let repeated = n.report(&["--json", "need", "sub"], 0, "already-done");
    assert_eq!(repeated.text("path"), "sub"); assert!(repeated.flag("gap"));
    assert_eq!(fs::read(n.folder.join("sub/keep")).unwrap(), b"keep\n"); assert_eq!(n.paths(), vec!["sub"]);
}
