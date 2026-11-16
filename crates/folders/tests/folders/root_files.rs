//! Top-level files given as folder paths (S-126, WO-folders-sparse-files) in a throwaway repository: make and its
//! repeat for request and zone folders, need, and remake_orphans hand git's cone-mode sparse_set / sparse_add only
//! directories, while the folder record keeps the paths exactly as given; a refill with only top-level files checks
//! out the top-level files alone; an orphan attempt whose record holds only top-level files is remade with its try ref
//! back, an empty cone, the top-level files alone and nothing deleted; an orphan whose folder is not sparse fails as
//! Git with its index and the op log unchanged, whether its cone is empty or holds a folder.
use super::attempt_support::{try_ref, Attempts};
use super::maintenance_support::records;
use crate::support::{run, setup, Fixture};
use folders::{make_attempts, make_request_folder, make_zone_folder, need, FolderError};
use gitio::Git;
use oplog::KillPidProbe;
use std::path::{Path, PathBuf};

/// A fixture with zone branch red at its head, and where its request or zone folder goes.
fn zone_red() -> (Fixture, PathBuf) {
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
    let path = f.repo.path.join("folder");
    (f, path)
}

fn paths(given: &[&str]) -> Vec<String> { given.iter().map(|path| (*path).into()).collect() }

fn local(f: &Fixture, folder: &Path) -> Git { Git { repo: folder.to_owned(), ..f.git.clone() } }

/// The folder's cone as `git sparse-checkout list` prints it, and its status is clean.
fn assert_cone(f: &Fixture, folder: &Path, list: &[u8]) {
    let local = local(f, folder);
    assert_eq!(String::from_utf8(run(&local, &["sparse-checkout", "list"])).unwrap(), std::str::from_utf8(list).unwrap());
    assert_eq!(String::from_utf8(run(&local, &["status", "--porcelain"])).unwrap(), "");
}

#[test]
fn a_request_folder_with_a_top_level_file_and_a_folder_refills_on_repeat_with_only_the_folder_in_its_cone() {
    let (f, path) = zone_red();
    let s = setup(&f.git, &path);
    let given = paths(&["Cargo.toml", "kit/zone"]);
    let made = make_request_folder(&s, "red", "one", &given).unwrap();
    let again = make_request_folder(&s, "red", "one", &given).unwrap();
    assert_eq!(again, made);
    assert_eq!(again.work_order_paths, ["Cargo.toml", "kit/zone"]);
    assert_cone(&f, &path, b"kit/zone\n");
    assert!(path.join("Cargo.toml").is_file());
    assert!(path.join("kit/zone/file").is_file());
    assert!(!path.join("other").exists());
}

#[test]
fn a_zone_folder_with_two_top_level_files_and_a_folder_refills_on_repeat_with_only_the_folder_in_its_cone() {
    let f = Fixture::new();
    let path = f.repo.path.join("folder");
    let s = setup(&f.git, &path);
    let given = paths(&["Cargo.toml", "README", "kit/zone"]);
    let made = make_zone_folder(&s, "red", &given, Some(&f.head)).unwrap();
    let again = make_zone_folder(&s, "red", &given, Some(&f.head)).unwrap();
    assert_eq!(again, made);
    assert_eq!(again.work_order_paths, ["Cargo.toml", "README", "kit/zone"]);
    assert_cone(&f, &path, b"kit/zone\n");
    assert!(path.join("Cargo.toml").is_file());
    assert!(path.join("README").is_file());
    assert!(path.join("kit/zone/file").is_file());
    assert!(!path.join("other").exists());
}

#[test]
fn a_request_folder_with_only_a_top_level_file_has_an_empty_cone_and_the_top_level_files_alone() {
    let (f, path) = zone_red();
    let s = setup(&f.git, &path);
    let given = paths(&["Cargo.toml"]);
    let made = make_request_folder(&s, "red", "one", &given).unwrap();
    let again = make_request_folder(&s, "red", "one", &given).unwrap();
    assert_eq!(again, made);
    assert_eq!(again.work_order_paths, ["Cargo.toml"]);
    assert_cone(&f, &path, b"");
    assert!(path.join("Cargo.toml").is_file());
    assert!(path.join("README").is_file());
    assert!(!path.join("kit").exists());
    assert!(!path.join("other").exists());
}

#[test]
fn need_of_a_top_level_file_is_recorded_but_leaves_the_cone_and_a_later_need_and_repeat_make_still_work() {
    let (f, path) = zone_red();
    let s = setup(&f.git, &path);
    let given = paths(&["Cargo.toml", "kit/zone"]);
    make_request_folder(&s, "red", "one", &given).unwrap();
    let root = need(&f.git, &path, "Cargo.toml", &KillPidProbe).unwrap();
    assert_eq!(root.needed_paths, ["Cargo.toml"]);
    assert_cone(&f, &path, b"kit/zone\n");
    let record = need(&f.git, &path, "needed", &KillPidProbe).unwrap();
    assert_eq!(record.needed_paths, ["Cargo.toml", "needed"]);
    assert_cone(&f, &path, b"kit/zone\nneeded\n");
    assert!(path.join("needed/file").is_file());
    assert_eq!(make_request_folder(&s, "red", "one", &given).unwrap(), record);
    assert_cone(&f, &path, b"kit/zone\nneeded\n");
}

#[test]
fn remake_of_an_orphan_whose_record_has_a_top_level_file_and_a_folder_remakes_it_with_nothing_outside_its_cone_deleted() {
    let a = Attempts::new();
    make_attempts(&setup(&a.f.git, &a.dir), "red", "one", &paths(&["Cargo.toml", "kit/zone"]), 1).unwrap();
    a.trysnap(1, true);
    a.orphan(1);
    assert_eq!(a.remake().unwrap(), vec![a.folder(1)]);
    assert!(a.f.git.read_ref(&try_ref(1)).unwrap().is_some());
    let status = String::from_utf8(run(&a.local(1), &["status", "--porcelain"])).unwrap();
    let deleted_outside = status.lines()
        .filter(|line| line.starts_with('D') || line.get(1..2) == Some("D"))
        .filter(|line| line.contains("other/") || line.contains("kit/dependency/"))
        .count();
    assert_eq!(deleted_outside, 0, "{status}");
}

#[test]
fn remake_of_an_orphan_whose_record_has_only_top_level_files_remakes_it_with_only_the_top_level_files_checked_out() {
    let a = Attempts::new();
    make_attempts(&setup(&a.f.git, &a.dir), "red", "one", &paths(&["Cargo.toml"]), 1).unwrap();
    a.trysnap(1, false);
    a.orphan(1);
    assert_eq!(a.remake().unwrap(), vec![a.folder(1)]);
    assert!(a.f.git.read_ref(&try_ref(1)).unwrap().is_some());
    let local = a.local(1);
    let status = String::from_utf8(run(&local, &["status", "--porcelain"])).unwrap();
    let deleted = status.lines().filter(|line| line.starts_with('D') || line.get(1..2) == Some("D")).count();
    assert_eq!(deleted, 0, "{status}");
    assert_eq!(String::from_utf8(run(&local, &["sparse-checkout", "list"])).unwrap(), "");
    assert!(a.folder(1).join("Cargo.toml").is_file());
    assert!(a.folder(1).join("README").is_file());
    assert!(!a.folder(1).join("kit").exists());
}

#[test]
fn remake_of_a_root_only_orphan_whose_folder_is_not_sparse_fails_as_git_with_nothing_changed() {
    assert_not_sparse_orphan_fails_as_git_with_nothing_changed(&["Cargo.toml"]);
}

#[test]
fn remake_of_an_orphan_with_a_folder_in_its_cone_whose_folder_is_not_sparse_fails_as_git_with_nothing_changed() {
    assert_not_sparse_orphan_fails_as_git_with_nothing_changed(&["Cargo.toml", "kit/zone"]);
}

/// Attempt 1 made with `given`, saved unchanged and orphaned, then made not sparse by hand: remake fails as Git with
/// the try ref still absent and the folder's index and the op log unchanged.
fn assert_not_sparse_orphan_fails_as_git_with_nothing_changed(given: &[&str]) {
    let a = Attempts::new();
    make_attempts(&setup(&a.f.git, &a.dir), "red", "one", &paths(given), 1).unwrap();
    a.trysnap(1, false);
    a.orphan(1);
    let local = a.local(1);
    run(&local, &["config", "--worktree", "--unset", "core.sparseCheckout"]);
    std::fs::remove_file(git_path(&local, "info/sparse-checkout")).unwrap();
    let index = git_path(&local, "index");
    let bytes = std::fs::read(&index).unwrap();
    let ops = records(&a.f).len();
    match a.remake() {
        Err(FolderError::Git(_)) => {}
        other => panic!("expected Git, got {other:?}"),
    }
    assert_eq!(a.f.git.read_ref(&try_ref(1)).unwrap(), None);
    assert_eq!(std::fs::read(&index).unwrap(), bytes);
    assert_eq!(records(&a.f).len(), ops);
}

/// `git rev-parse --git-path <name>` in the folder, resolved against the folder when git prints it relative.
fn git_path(local: &Git, name: &str) -> PathBuf {
    let printed = String::from_utf8(run(local, &["rev-parse", "--git-path", name])).unwrap();
    local.repo.join(printed.trim_end())
}
