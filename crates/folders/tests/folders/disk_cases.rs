//! #1019: disk/sweep_build -> real worktree discovery, file/symlink sizes and controlled filesystem mtimes.
use super::maintenance_support::*;
use crate::support::{run, setup, write};
use folders::{disk, make_request_folder, make_zone_folder, sweep_build};
use std::{fs::{self, File, FileTimes}, os::unix::fs::symlink, path::Path, time::{Duration, UNIX_EPOCH}};

#[test]
fn disk_lists_only_live_mortar_folders_measures_literal_bytes_and_mtimes_and_counts_shared_store_once() {
    let m = Removal::new();
    let before_store = disk(&m.f.git, 1, &[], &[]).unwrap().store_bytes;
    assert!(before_store > 0, "the fixture has real objects");
    let zone = m.f.repo.path.join("standing");
    make_zone_folder(&setup(&m.f.git, &zone), "red", &["kit/zone".into()], None).unwrap();
    let gone = m.f.repo.path.join("prunable");
    make_request_folder(&setup(&m.f.git, &gone), "red", "ghost", &["kit/zone".into()]).unwrap();
    fs::remove_dir_all(&gone).unwrap();
    assert!(m.f.git.worktree_list().unwrap().iter().any(|w| w.path == gone && w.prunable));
    write(&zone, "local", b"abc");
    write(&zone, "nested/target/file", b"12345");
    write(&zone, "nested/.git/meta", b"meta");
    write(&zone, "target/build", b"build!!");
    write(&m.f.repo.path, "external-large", &[b'x'; 1024]);
    symlink("../external-large", zone.join("link")).unwrap();
    stamp_tree(&zone, 1_000_000_000);
    stamp_tree(&m.folder, 1_000_000_000);
    stamp(&zone.join("local"), 2_000_000_000);
    stamp(&zone.join("target/build"), 2_100_000_000);
    for name in ["refs/mortar/snap/red/closed", SNAP, "refs/tags/v1"] { run(&m.f.git, &["update-ref", name, m.f.head.as_str()]); }
    fs::write(m.f.common.join("objects/integration-accounting"), b"thirteen-byte").unwrap();
    let before_log = log(&m.f); let before_refs = refs(&m.f.git);
    let report = disk(&m.f.git, 1, &[], &[]).unwrap();
    let mut names = report.folders.iter().map(|f| f.path.clone()).collect::<Vec<_>>(); names.sort();
    let mut expected = vec![m.folder.clone(), zone.clone()]; expected.sort();
    assert_eq!(names, expected, "neither main checkout nor prunable worktree is a Mortar folder");
    let z = report.folders.iter().find(|f| f.path == zone).unwrap();
    // Root fixture files: Cargo.toml18 + README10 + kit/zone/file5 + conflict5 =38.
    // Added data3 + nested target5 + nested .git4 + symlink's own17 =29. External1024 is not followed.
    assert_eq!(z.files_bytes, 67); assert_eq!(z.build_bytes, 7); assert_eq!(z.newest, Some(2_100_000_000));
    assert_eq!(z.record.kind, folders::record::FolderKind::Zone); assert_eq!(z.record.zone, "red");
    assert_eq!(z.record.request, None); assert_eq!(z.record.base, m.f.head);
    let req = report.folders.iter().find(|f| f.path == m.folder).unwrap();
    assert_eq!(req.files_bytes, 38); assert_eq!(req.build_bytes, 0); assert_eq!(req.newest, Some(1_000_000_000));
    assert_eq!(req.record.kind, folders::record::FolderKind::Request); assert_eq!(req.record.zone, "red");
    assert_eq!(req.record.request.as_deref(), Some("one")); assert_eq!(req.record.base, m.f.head);
    assert_eq!(report.store_bytes, before_store + 13, "adding linked folders must not multiply shared object bytes");
    assert_eq!(report.refs, vec![("refs/heads".into(), 1), ("refs/heads/req".into(), 2), ("refs/heads/zone".into(), 1), ("refs/mortar/snap".into(), 2), ("refs/tags".into(), 1)]);
    assert_eq!(report.sweep.bytes, 0, "old request has no target; future-touched zone is not idle");
    assert_eq!(refs(&m.f.git), before_refs); assert_eq!(log(&m.f), before_log);
    assert_eq!(fs::read(zone.join("link")).unwrap(), vec![b'x'; 1024]);
}

#[test]
fn disk_sweep_figure_matches_exact_idle_build_deletion_while_recent_folder_main_and_other_files_survive() {
    let m = Removal::new();
    let zone = m.f.repo.path.join("idle-zone");
    make_zone_folder(&setup(&m.f.git, &zone), "red", &["kit/zone".into()], None).unwrap();
    write(&zone, "target/build", b"1234567"); write(&zone, "nested/target/keep", b"ok");
    write(&m.folder, "target/build", b"12345"); write(&m.f.repo.path, "target/main-build", b"main stays\n");
    stamp_tree(&zone, 1_000_000_000); stamp_tree(&m.folder, 1_000_000_000);
    stamp(&m.folder.join("target/build"), 2_000_000_000); // A build-file touch alone keeps the folder active (S-52).
    let before_refs = refs(&m.f.git); let before_log = log(&m.f);
    let report = disk(&m.f.git, 1, &[], &[]).unwrap(); assert_eq!(report.sweep.bytes, 7);
    let result = sweep_build(&m.f.git, 1).unwrap();
    assert_eq!(result.removed, vec![zone.join("target")]); assert_eq!(result.freed_bytes, 7);
    assert_eq!(result.freed_bytes, report.sweep.bytes);
    assert!(!zone.join("target").exists());
    assert_eq!(fs::read(zone.join("nested/target/keep")).unwrap(), b"ok");
    assert_eq!(fs::read(zone.join("kit/zone/file")).unwrap(), b"zone\n");
    assert_eq!(fs::read(m.folder.join("target/build")).unwrap(), b"12345");
    assert_eq!(fs::read(m.f.repo.path.join("target/main-build")).unwrap(), b"main stays\n");
    assert_eq!(refs(&m.f.git), before_refs); assert_eq!(log(&m.f), before_log);
    let repeat = sweep_build(&m.f.git, 1).unwrap(); assert!(repeat.removed.is_empty()); assert_eq!(repeat.freed_bytes, 0);
}

#[test]
fn sweep_never_follows_or_deletes_a_symlink_build_dir_even_when_selected_as_idle() {
    let m = Removal::new();
    write(&m.f.repo.path, "outside-target/keep", b"external build stays\n");
    symlink("../outside-target", m.folder.join("target")).unwrap();
    let result = sweep_build(&m.f.git, 0).unwrap();
    assert!(result.removed.is_empty()); assert_eq!(result.freed_bytes, 0);
    assert!(fs::symlink_metadata(m.folder.join("target")).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(m.f.repo.path.join("outside-target/keep")).unwrap(), b"external build stays\n");
    let report = disk(&m.f.git, 0, &[], &[]).unwrap();
    assert_eq!(report.folders[0].build_bytes, 17, "symlink length, not the target's bytes");
}

#[test]
fn empty_recorded_worktree_has_zero_files_and_missing_build_has_zero_bytes_and_no_newest_time() {
    let m = Removal::new();
    // Remove only this throwaway fixture's own working files, leaving its real .git pointer and record.
    for item in fs::read_dir(&m.folder).unwrap() {
        let item = item.unwrap(); if item.file_name() == ".git" { continue; }
        if item.file_type().unwrap().is_dir() { fs::remove_dir_all(item.path()).unwrap(); }
        else { fs::remove_file(item.path()).unwrap(); }
    }
    let report = disk(&m.f.git, 1, &[], &[]).unwrap();
    assert_eq!(report.folders.len(), 1); assert_eq!(report.folders[0].path, m.folder);
    assert_eq!(report.folders[0].files_bytes, 0); assert_eq!(report.folders[0].build_bytes, 0);
    assert_eq!(report.folders[0].newest, None);
}

fn stamp(path: &Path, seconds: u64) {
    File::open(path).unwrap().set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds))).unwrap();
}
fn stamp_tree(root: &Path, seconds: u64) {
    for item in fs::read_dir(root).unwrap() {
        let item = item.unwrap(); if item.file_name() == ".git" { continue; }
        let kind = item.file_type().unwrap(); if kind.is_symlink() { continue; }
        if kind.is_dir() { stamp_tree(&item.path(), seconds); }
        stamp(&item.path(), seconds);
    }
}
