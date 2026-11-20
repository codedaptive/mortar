use crate::{json::check, support::*};
use std::fs;

#[test]
fn save_child_json_has_real_commit_id_trailers_and_inert_repeat() {
    let f = Fixture::new(); f.write("src/file", b"saved binary output\n");
    // Running within the folder still addresses the request's top and shared config/lock.
    let report = check(f.command(&f.request.join("src"), &["--json", "save", "Save from binary"]).output().unwrap(), "save", 0, "saved");
    let head = f.head(); let info = f.git.commit_info(&head).unwrap();
    assert_eq!(report.text("commit"), head.as_str()); assert!(report.flag("written"));
    assert_eq!(report.text("next"), "mr join"); assert!(report.strings("refused").is_empty()); assert!(report.is_null("cap"));
    assert_eq!(info.parents, vec![f.base.clone()]); assert!(info.message.starts_with("Save from binary\n\n"));
    let trailers = oplog::read_mortar_trailers(&f.git, &info.message).unwrap();
    let id = trailers.change_id.unwrap(); assert_eq!(report.text("change_id"), id.as_str());
    assert_eq!(id.as_str().len(), 32); assert!(id.as_str().bytes().all(|b| (b'k'..=b'z').contains(&b)));
    assert_eq!(trailers.zone.as_deref(), Some("core")); assert_eq!(trailers.request.as_deref(), Some("task"));
    assert_eq!(f.blob(&head, "src/file"), b"saved binary output\n");
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty()); assert!(!f.lock_path().exists());
    let before = f.observe();
    let repeat = f.report(&["--json", "save", "No new work"], "save", 0, "unchanged");
    assert!(!repeat.flag("written")); assert!(repeat.is_null("change_id")); assert_eq!(repeat.text("commit"), head.as_str());
    assert_eq!(f.observe(), before);
}

#[test]
fn attempt_save_uses_trysnap_and_keeps_request_branch_snapshot_and_change_id() {
    let f = Fixture::new(); let request = f.save_topic();
    let snap = f.git.read_ref(SNAP).unwrap();
    let attempts = f.root.join("attempts");
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_attempts(&setup(&f.git, &attempts), "core", "task", &["src".into(), "sub".into()], 1).unwrap();
    let path = attempts.join("try-1"); write(&path, "src/file", b"attempt saved output\n");
    let report = check(f.command(&path, &["--json", "save", "Save attempt"]).output().unwrap(), "save", 0, "saved");
    let attempt = f.git.read_ref("refs/heads/try/core/task/1").unwrap().unwrap();
    let try_snap = f.git.read_ref("refs/mortar/trysnap/core/task/1").unwrap().unwrap();
    assert_eq!(report.text("commit"), attempt.as_str()); assert_eq!(f.head(), request);
    assert_eq!(f.git.read_ref(SNAP).unwrap(), snap);
    assert_eq!(f.blob(&attempt, "src/file"), b"attempt saved output\n");
    assert_eq!(f.blob(&try_snap, "src/file"), b"attempt saved output\n");
    let original = oplog::read_mortar_trailers(&f.git, &f.git.commit_info(&request).unwrap().message).unwrap().change_id.unwrap();
    assert_eq!(report.text("change_id"), original.as_str()); assert!(!f.lock_path().exists());
}

#[test]
fn blank_or_two_line_save_sentence_refuses_without_lock_snapshot_commit_or_log_changes() {
    for sentence in ["   ", "one\ntwo"] {
        let f = Fixture::new(); f.write("src/file", b"must remain unsaved\n");
        let before = f.observe();
        let report = f.report(&["--json", "save", sentence], "save", 1, "refused");
        assert!(report.lines().contains("nothing ran")); assert!(report.text("next").contains("mr save"));
        assert_eq!(f.observe(), before); assert_eq!(f.git.read_ref(SNAP).unwrap(), None); assert!(!f.lock_path().exists());
    }
}

#[test]
fn zone_save_and_missing_config_refuse_before_any_work() {
    let f = Fixture::new(); let before = f.observe();
    let zone = check(f.command(&f.zone, &["--json", "save", "Do not save zone"]).output().unwrap(), "save", 1, "refused");
    assert!(zone.lines().contains("zone folder")); assert_eq!(f.observe(), before);
    fs::remove_file(f.common.join("mortar/config.json")).unwrap();
    let missing = f.report(&["--json", "save", "Needs config"], "save", 1, "no-config");
    assert_eq!(missing.text("next"), "mr init"); assert_eq!(f.observe(), before); assert!(!f.lock_path().exists());
}

#[test]
fn save_lists_new_over_cap_file_with_literal_size_and_saves_tracked_changes() {
    let f = Fixture::new(); let mut config = f.init_args.clone(); config.cap_bytes = 3; f.configure(&config);
    f.write("src/large", b"1234567"); f.write("src/file", b"tracked change stays despite cap\n");
    let report = f.report(&["--json", "save", "Save capped work"], "save", 0, "saved");
    assert_eq!(report.strings("refused"), &["src/large"]); assert_eq!(report.number("cap"), 3);
    assert!(report.lines().contains("7 bytes, over the size cap of 3 bytes"));
    let head = f.head(); assert_eq!(f.blob(&head, "src/file"), b"tracked change stays despite cap\n");
    assert_eq!(f.git.tree_entry(&f.git.commit_info(&head).unwrap().tree, "src/large".as_ref()).unwrap(), None);
    assert_eq!(fs::read(f.request.join("src/large")).unwrap(), b"1234567");
}

#[test]
fn save_and_join_exclude_top_level_and_nested_configured_build_dirs_without_deleting_them() {
    for verb in ["save", "join"] {
        let f = Fixture::new();
        f.write("src/file", b"included change\n"); f.write("sub/positive", b"included neighbor\n");
        f.write("d/top-build", b"root build output\n"); f.write("sub/d/nested-build", b"nested build output\n");
        let args = if verb == "save" { vec!["--json", "save", "Save without build output"] } else { vec!["--json", "join"] };
        let outcome = if verb == "save" { "saved" } else { "joined" };
        let report = f.report(&args, verb, 0, outcome); assert!(report.strings("refused").is_empty());
        for id in [f.head(), f.git.read_ref(SNAP).unwrap().unwrap()] {
            assert_eq!(f.blob(&id, "src/file"), b"included change\n"); assert_eq!(f.blob(&id, "sub/positive"), b"included neighbor\n");
            let names = raw(&f.git, &["ls-tree", "-r", "--name-only", id.as_str()]);
            assert_eq!(names, b"root.txt\nsrc/file\nsub/keep\nsub/positive\n", "{verb}: only literal tracked and included paths");
        }
        assert_eq!(fs::read(f.request.join("d/top-build")).unwrap(), b"root build output\n");
        assert_eq!(fs::read(f.request.join("sub/d/nested-build")).unwrap(), b"nested build output\n");
    }
}
