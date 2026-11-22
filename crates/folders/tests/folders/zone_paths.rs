//! set_zone_paths on real zone folders in a throwaway repository: widening and narrowing replace the record's
//! work-order paths, clear its needed paths and set the cone, and a repeat with the same paths changes nothing; a
//! request folder, a plain worktree and a folder with unmerged entries are refused with nothing changed; a modified
//! tracked file and an untracked file under a dropped path stay on disk.
use crate::support::{run, setup, write, Fixture};
use folders::{make_request_folder, make_zone_folder, need, set_zone_paths, FolderError, FolderKind, FolderRecord};
use folders::make::read_folder_record;
use folders::record::record_path;
use gitio::{Git, RunOpts};
use oplog::KillPidProbe;
use std::fs;
use std::path::{Path, PathBuf};

fn paths(given: &[&str]) -> Vec<String> { given.iter().map(|path| (*path).into()).collect() }

fn local(f: &Fixture, folder: &Path) -> Git { Git { repo: folder.to_owned(), ..f.git.clone() } }

/// A fixture with zone folder red made at its head with `given`, and the folder's path.
fn zone_folder(given: &[&str]) -> (Fixture, PathBuf) {
    let f = Fixture::new();
    let path = f.repo.path.join("folder");
    make_zone_folder(&setup(&f.git, &path), "red", &paths(given), Some(&f.head)).unwrap();
    (f, path)
}

/// The folder's cone as `git sparse-checkout list` prints it.
fn cone(f: &Fixture, folder: &Path) -> String {
    String::from_utf8(run(&local(f, folder), &["sparse-checkout", "list"])).unwrap()
}

fn lock_free(f: &Fixture) -> bool { !f.common.join("mortar/locks/folders.lock").exists() }

struct Row {
    case: &'static str,
    made: &'static [&'static str],
    given: &'static [&'static str],
    recorded: &'static [&'static str],
    cone: &'static str,
    present: &'static [(&'static str, &'static str)],
    absent: &'static [&'static str],
}

#[test]
fn set_zone_paths_replaces_the_paths_clears_needed_sets_the_cone_and_a_repeat_changes_nothing() {
    let rows = [
        Row {
            case: "widen", made: &["kit/zone"], given: &["needed", "kit/dependency", "kit/zone/", "kit/zone"],
            recorded: &["kit/dependency", "kit/zone", "needed"], cone: "kit/dependency\nkit/zone\nneeded\n",
            present: &[("kit/dependency/file", "dependency\n"), ("kit/zone/file", "zone\n"), ("needed/file", "needed\n")],
            absent: &["other/file"],
        },
        Row {
            case: "narrow", made: &["kit/dependency", "kit/zone"], given: &["kit/zone", "Cargo.toml"],
            recorded: &["Cargo.toml", "kit/zone"], cone: "kit/zone\n",
            present: &[("kit/zone/file", "zone\n"), ("Cargo.toml", "# root build file\n")],
            absent: &["kit/dependency/file", "needed/file", "other/file"],
        },
    ];
    for row in rows {
        let (f, path) = zone_folder(row.made);
        assert_eq!(need(&f.git, &path, "needed", &KillPidProbe).unwrap().needed_paths, ["needed"], "{}", row.case);
        let expected = FolderRecord {
            kind: FolderKind::Zone, zone: "red".into(), request: None, base: f.head.clone(),
            work_order_paths: paths(row.recorded), needed_paths: vec![], parent: None,
        };
        let record = set_zone_paths(&f.git, &path, &paths(row.given), &KillPidProbe).unwrap();
        assert_eq!(record, expected, "{}", row.case);
        let git_dir = f.git.git_dir(&path).unwrap();
        assert_eq!(read_folder_record(&git_dir).unwrap(), Some(expected), "{}", row.case);
        assert_folder(&f, &path, &row);
        let record_bytes = fs::read(record_path(&git_dir)).unwrap();
        let index = fs::read(git_dir.join("index")).unwrap();
        assert_eq!(set_zone_paths(&f.git, &path, &paths(row.given), &KillPidProbe).unwrap(), record, "{}", row.case);
        assert_eq!(fs::read(record_path(&git_dir)).unwrap(), record_bytes, "{}", row.case);
        assert_eq!(fs::read(git_dir.join("index")).unwrap(), index, "{}", row.case);
        assert_folder(&f, &path, &row);
        assert!(lock_free(&f), "{}", row.case);
    }
}

/// The folder's cone, files and clean status are the row's.
fn assert_folder(f: &Fixture, path: &Path, row: &Row) {
    assert_eq!(cone(f, path), row.cone, "{}", row.case);
    for (file, content) in row.present {
        assert_eq!(fs::read_to_string(path.join(file)).unwrap(), *content, "{}: {file}", row.case);
    }
    for file in row.absent { assert!(!path.join(file).exists(), "{}: {file}", row.case); }
    assert_eq!(String::from_utf8(run(&local(f, path), &["status", "--porcelain"])).unwrap(), "", "{}", row.case);
}

#[test]
fn work_under_a_dropped_path_stays_on_disk_after_a_narrow() {
    let (f, path) = zone_folder(&["kit/dependency", "kit/zone"]);
    write(&path, "kit/dependency/file", b"unsaved work\n");
    write(&path, "kit/dependency/untracked", b"untracked work\n");
    let record = set_zone_paths(&f.git, &path, &paths(&["kit/zone"]), &KillPidProbe).unwrap();
    assert_eq!(record.work_order_paths, ["kit/zone"]);
    assert_eq!(cone(&f, &path), "kit/zone\n");
    assert_eq!(fs::read(path.join("kit/dependency/file")).unwrap(), b"unsaved work\n");
    assert_eq!(fs::read(path.join("kit/dependency/untracked")).unwrap(), b"untracked work\n");
}

#[test]
fn a_zone_folder_with_unmerged_entries_is_refused_as_not_saved_with_record_cone_and_files_unchanged() {
    let (f, path) = zone_folder(&["kit/zone"]);
    let zone = local(&f, &path);
    write(&path, "kit/zone/conflict", b"zone side\n");
    run(&zone, &["add", "kit/zone/conflict"]);
    run(&zone, &["commit", "-m", "zone change"]);
    write(&f.repo.path, "kit/zone/conflict", b"other side\n");
    run(&f.git, &["add", "kit/zone/conflict"]);
    run(&f.git, &["commit", "-m", "other change"]);
    let other = f.git.rev_parse("HEAD").unwrap().unwrap();
    let out = zone.run(&["merge", other.as_str()], &RunOpts { answer_codes: vec![1], ..RunOpts::default() }).unwrap();
    assert_eq!(out.status, 1);
    assert!(!run(&zone, &["ls-files", "--unmerged"]).is_empty());
    let git_dir = f.git.git_dir(&path).unwrap();
    let record = fs::read(record_path(&git_dir)).unwrap();
    let index = fs::read(git_dir.join("index")).unwrap();
    let conflict = fs::read(path.join("kit/zone/conflict")).unwrap();
    match set_zone_paths(&f.git, &path, &paths(&["kit/dependency", "kit/zone"]), &KillPidProbe) {
        Err(FolderError::NotSaved { path: actual }) => assert_eq!(actual, path),
        other => panic!("expected NotSaved: {other:?}"),
    }
    assert_eq!(fs::read(record_path(&git_dir)).unwrap(), record);
    assert_eq!(fs::read(git_dir.join("index")).unwrap(), index);
    assert_eq!(fs::read(path.join("kit/zone/conflict")).unwrap(), conflict);
    assert_eq!(cone(&f, &path), "kit/zone\n");
    assert!(!path.join("kit/dependency/file").exists());
    assert!(lock_free(&f));
}

#[test]
fn a_request_folder_is_refused_as_a_different_folder_with_record_and_cone_unchanged() {
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let path = f.repo.path.join("folder");
    let made = make_request_folder(&setup(&f.git, &path), "red", "one", &paths(&["kit/zone"])).unwrap();
    let git_dir = f.git.git_dir(&path).unwrap();
    let record = fs::read(record_path(&git_dir)).unwrap();
    match set_zone_paths(&f.git, &path, &paths(&["kit/dependency", "kit/zone"]), &KillPidProbe) {
        Err(FolderError::DifferentFolder { path: actual, existing }) => {
            assert_eq!(actual, path);
            assert_eq!(existing, made);
        }
        other => panic!("expected DifferentFolder: {other:?}"),
    }
    assert_eq!(fs::read(record_path(&git_dir)).unwrap(), record);
    assert_eq!(cone(&f, &path), "kit/zone\n");
    assert!(!path.join("kit/dependency/file").exists());
    assert!(lock_free(&f));
}

#[test]
fn a_plain_worktree_with_no_record_is_refused_as_not_a_folder_without_a_record_written() {
    let f = Fixture::new();
    let plain = f.repo.path.join("plain");
    f.git.worktree_add(&plain, &f.head, None).unwrap();
    let result = set_zone_paths(&f.git, &plain, &paths(&["kit/zone"]), &KillPidProbe);
    assert!(matches!(result, Err(FolderError::NotAFolder { .. })), "{result:?}");
    assert!(!record_path(&f.git.git_dir(&plain).unwrap()).exists());
    assert!(!plain.join("kit/zone/file").exists());
    assert!(lock_free(&f));
}
