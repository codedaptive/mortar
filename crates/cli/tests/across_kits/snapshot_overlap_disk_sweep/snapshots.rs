//! CLI run_snapshot -> actual Snapshot formatted/capped sparse capture -> Git refs/trees and Oplog.
use super::fixture::*;
use crate::support::{raw, write, SNAP};
use gitio::Oid;
use std::fs;

#[test]
fn sparse_request_snapshot_formats_keeps_omitted_paths_excludes_builds_caps_new_files_and_repeats_inertly() {
    let w = World::new(); let path = w.s.f.root.join("sparse");
    app(&w, &["folder", "request", "core", "sparse", path.to_str().unwrap(), "--paths", "src"], 0, "made");
    assert!(!path.join("sub").exists());
    let mut config = w.s.f.init_args.clone(); config.build_dirs = vec!["target/".into()]; config.cap_bytes = 8;
    config.formatter = Some(crate::support::FormatterConfig {
        command: vec!["/bin/sh".into(), "-c".into(), "printf 'formatted\\n' > src/file".into()], timeout_seconds: 30,
    });
    w.s.f.configure(&config); write(&path, "src/file", b"raw\n"); write(&path, "src/large", b"123456789");
    write(&path, "target/output", b"build stays\n");
    let branch = ref_at(&w, "refs/heads/req/core/sparse"); let zone = w.tip("core");
    let r = run(&w, &path.join("src"), &["snapshot", "--key", "holder-only-snapshot"], 0, "snapshotted");
    let id = ref_at(&w, "refs/mortar/snap/core/sparse").unwrap();
    assert_eq!(r.text("commit"), id.as_str()); assert!(r.flag("written"));
    let info = w.s.f.git.commit_info(&id).unwrap(); assert_eq!(r.text("tree"), info.tree.as_str());
    assert_eq!(w.s.f.blob(&id, "src/file"), b"formatted\n"); assert_eq!(w.s.f.blob(&id, "sub/keep"), b"keep\n");
    assert_eq!(raw(&w.s.f.git, &["ls-tree", "-r", "--name-only", id.as_str()]), b"root.txt\nsrc/file\nsub/keep\n");
    assert_eq!(r.strings("refused"), &["src/large"]);
    assert!(r.strings("lines").contains(&"not snapshotted: src/large is 9 bytes, over the size cap of 8 bytes".into()));
    assert!(r.strings("lines").iter().any(|line| line.ends_with(": formatted")));
    assert_eq!(fs::read(path.join("target/output")).unwrap(), b"build stays\n");
    assert_eq!(fs::read(path.join("src/large")).unwrap(), b"123456789");
    assert_eq!(ref_at(&w, "refs/heads/req/core/sparse"), branch); assert_eq!(w.tip("core"), zone);
    assert!(!w.s.f.lock_path().exists()); assert_eq!(r.text("next"), "nothing to do");
    let before = observe(&w);
    let repeat = run(&w, &path, &["snapshot", "--key", "holder-only-snapshot"], 0, "unchanged");
    assert!(!repeat.flag("written")); assert_eq!(repeat.text("commit"), id.as_str()); assert_eq!(observe(&w), before);
}

#[test]
fn attempt_snapshot_moves_only_its_trysnap_and_never_the_request_or_attempt_branch() {
    let w = World::new(); w.s.f.write("src/file", b"request unsaved\n");
    run(&w, &w.s.f.request, &["snapshot"], 0, "snapshotted");
    let request_snap = ref_at(&w, SNAP); let request_tip = w.s.f.head();
    let attempts = w.s.f.root.join("attempts");
    run(&w, &w.s.f.request, &["folder", "attempts", "1", attempts.to_str().unwrap()], 0, "made");
    let path = attempts.join("try-1"); let branch = ref_at(&w, "refs/heads/try/core/task/1");
    write(&path, "src/file", b"attempt unsaved\n");
    let r = run(&w, &path, &["snapshot"], 0, "snapshotted");
    let id = ref_at(&w, "refs/mortar/trysnap/core/task/1").unwrap();
    assert_eq!(r.text("commit"), id.as_str()); assert_eq!(w.s.f.blob(&id, "src/file"), b"attempt unsaved\n");
    assert_eq!(ref_at(&w, SNAP), request_snap); assert_eq!(w.s.f.head(), request_tip);
    assert_eq!(ref_at(&w, "refs/heads/try/core/task/1"), branch); assert!(!w.s.f.lock_path().exists());
    let before = observe(&w); let repeat = run(&w, &path, &["snapshot"], 0, "unchanged");
    assert_eq!(Oid::parse(repeat.text("commit")).unwrap(), id); assert_eq!(observe(&w), before);
}

#[test]
fn snapshot_zone_folder_and_non_mortar_checkout_refuse_without_mutation() {
    let w = World::new(); let before = observe(&w);
    let r = run(&w, &w.s.f.zone, &["snapshot"], 3, "refused");
    assert_eq!(r.strings("lines"), &[format!("mr snapshot runs in a request or attempt folder; {} is a zone folder", w.s.f.zone.display())]);
    assert_eq!(r.text("next"), "run mr snapshot --actor <name> in a request or attempt folder");
    assert_eq!(observe(&w), before);
    let r = app(&w, &["snapshot"], 3, "not-a-folder");
    assert_eq!(r.strings("lines"), &[format!("this directory is not in a Mortar folder: the worktree {} has no Mortar record", w.s.f.git.repo.display())]);
    assert_eq!(r.text("owner"), "the harness that makes Mortar folders");
    assert_eq!(r.text("next"), "run mr snapshot inside a folder Mortar made"); assert_eq!(observe(&w), before);
}
