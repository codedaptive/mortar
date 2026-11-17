//! first_on_path: the first PATH entry holding an executable regular file (or a symlink to one) with the name, on
//! files each test makes in its own scratch folder under the system temp dir.

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};

use super::first_on_path;

/// A scratch folder unique to one test in this process, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(test: &str) -> Scratch {
        let root = std::env::temp_dir().join(format!("cli-first-on-path-{}-{test}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Scratch(root)
    }

    fn dir(&self, name: &str) -> PathBuf {
        let dir = self.0.join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn file(dir: &Path, name: &str, mode: u32) {
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
}

fn path_of(dirs: &[&str]) -> OsString {
    OsString::from(dirs.join(":"))
}

#[test]
fn the_first_of_two_dirs_holding_an_executable_wins() {
    let scratch = Scratch::new("first-of-two");
    let (a, b) = (scratch.dir("a"), scratch.dir("b"));
    file(&a, "mr", 0o755);
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", &path), Some(a.join("mr")));
}

#[test]
fn a_non_executable_file_is_skipped() {
    let scratch = Scratch::new("non-executable");
    let (a, b) = (scratch.dir("a"), scratch.dir("b"));
    file(&a, "mr", 0o644);
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", &path), Some(b.join("mr")));
}

#[test]
fn only_non_executables_give_none() {
    let scratch = Scratch::new("only-non-executable");
    let a = scratch.dir("a");
    file(&a, "mr", 0o644);
    assert_eq!(first_on_path("mr", &path_of(&[a.to_str().unwrap()])), None);
}

#[test]
fn empty_entries_are_skipped() {
    let scratch = Scratch::new("empty-entries");
    let a = scratch.dir("a");
    file(&a, "mr", 0o755);
    let path = path_of(&["", "", a.to_str().unwrap(), ""]);
    assert_eq!(first_on_path("mr", &path), Some(a.join("mr")));
}

#[test]
fn an_empty_path_gives_none() {
    assert_eq!(first_on_path("mr", &OsString::new()), None);
}

#[test]
fn a_symlink_to_an_executable_counts_and_is_named_by_its_path_entry() {
    let scratch = Scratch::new("symlink-executable");
    let (a, real) = (scratch.dir("a"), scratch.dir("real"));
    file(&real, "mortar", 0o755);
    symlink(real.join("mortar"), a.join("mr")).unwrap();
    let path = path_of(&[a.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", &path), Some(a.join("mr")));
}

#[test]
fn a_symlink_to_a_non_executable_is_skipped() {
    let scratch = Scratch::new("symlink-non-executable");
    let (a, b, real) = (scratch.dir("a"), scratch.dir("b"), scratch.dir("real"));
    file(&real, "notes", 0o644);
    symlink(real.join("notes"), a.join("mr")).unwrap();
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", &path), Some(b.join("mr")));
}

#[test]
fn a_dangling_symlink_is_skipped() {
    let scratch = Scratch::new("dangling");
    let a = scratch.dir("a");
    symlink(scratch.0.join("gone"), a.join("mr")).unwrap();
    assert_eq!(first_on_path("mr", &path_of(&[a.to_str().unwrap()])), None);
}

#[test]
fn a_directory_with_the_name_is_skipped() {
    let scratch = Scratch::new("directory");
    let (a, b) = (scratch.dir("a"), scratch.dir("b"));
    fs::create_dir(a.join("mr")).unwrap();
    fs::set_permissions(a.join("mr"), fs::Permissions::from_mode(0o755)).unwrap();
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", &path), Some(b.join("mr")));
}

#[test]
fn an_executable_with_another_name_does_not_match() {
    let scratch = Scratch::new("other-name");
    let a = scratch.dir("a");
    file(&a, "mortar", 0o755);
    file(&a, "mrx", 0o755);
    assert_eq!(first_on_path("mr", &path_of(&[a.to_str().unwrap()])), None);
}

#[test]
fn a_dir_that_does_not_exist_is_skipped() {
    let scratch = Scratch::new("missing-dir");
    let a = scratch.dir("a");
    file(&a, "mr", 0o755);
    let missing = scratch.0.join("missing");
    let path = path_of(&[missing.to_str().unwrap(), a.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", &path), Some(a.join("mr")));
}
