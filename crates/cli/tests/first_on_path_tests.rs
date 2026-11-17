//! first_on_path: the first PATH entry holding an executable regular file (or a symlink to one) with the name, an
//! empty entry read as the given current directory and a relative one under it, and an unset PATH searching nothing,
//! on files each test makes in its own scratch folder under the system temp dir. The current directory is passed as data; no test changes the process's.
//! ZIT (rule 11, S-104): it calls cli::path::first_on_path on real directories, files with set permissions and
//! symlinks it makes itself in that folder and removes after the test.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};

use cli::path::first_on_path;

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
    let (a, b, cwd) = (scratch.dir("a"), scratch.dir("b"), scratch.dir("cwd"));
    file(&a, "mr", 0o755);
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(a.join("mr")));
}

#[test]
fn a_non_executable_file_is_skipped() {
    let scratch = Scratch::new("non-executable");
    let (a, b, cwd) = (scratch.dir("a"), scratch.dir("b"), scratch.dir("cwd"));
    file(&a, "mr", 0o644);
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(b.join("mr")));
}

#[test]
fn only_non_executables_give_none() {
    let scratch = Scratch::new("only-non-executable");
    let (a, cwd) = (scratch.dir("a"), scratch.dir("cwd"));
    file(&a, "mr", 0o644);
    assert_eq!(first_on_path("mr", Some(path_of(&[a.to_str().unwrap()]).as_os_str()), &cwd), None);
}

#[test]
fn empty_entries_without_the_name_in_cwd_fall_through_to_the_next_entry() {
    let scratch = Scratch::new("empty-entries");
    let (a, cwd) = (scratch.dir("a"), scratch.dir("cwd"));
    file(&a, "mr", 0o755);
    let path = path_of(&["", "", a.to_str().unwrap(), ""]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(a.join("mr")));
}

#[test]
fn a_leading_empty_entry_finds_the_cwd_executable_before_a_later_dir() {
    let scratch = Scratch::new("leading-empty");
    let (safe, cwd) = (scratch.dir("safe"), scratch.dir("cwd"));
    file(&cwd, "mr", 0o755);
    file(&safe, "mr", 0o755);
    let path = path_of(&["", safe.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(cwd.join("mr")));
}

#[test]
fn a_trailing_empty_entry_finds_the_cwd_executable_when_no_dir_has_one() {
    let scratch = Scratch::new("trailing-empty");
    let (a, cwd) = (scratch.dir("a"), scratch.dir("cwd"));
    file(&cwd, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), ""]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(cwd.join("mr")));
}

#[test]
fn a_relative_entry_is_looked_up_under_cwd() {
    let scratch = Scratch::new("relative");
    let cwd = scratch.dir("cwd");
    let sub = scratch.dir("cwd/sub");
    file(&sub, "mr", 0o755);
    assert_eq!(first_on_path("mr", Some(path_of(&["sub"]).as_os_str()), &cwd), Some(cwd.join("sub").join("mr")));
}

#[test]
fn an_empty_path_with_no_executable_in_cwd_gives_none() {
    let scratch = Scratch::new("empty-path");
    let cwd = scratch.dir("cwd");
    assert_eq!(first_on_path("mr", Some(OsStr::new("")), &cwd), None);
}

#[test]
fn a_symlink_to_an_executable_counts_and_is_named_by_its_path_entry() {
    let scratch = Scratch::new("symlink-executable");
    let (a, real, cwd) = (scratch.dir("a"), scratch.dir("real"), scratch.dir("cwd"));
    file(&real, "mortar", 0o755);
    symlink(real.join("mortar"), a.join("mr")).unwrap();
    let path = path_of(&[a.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(a.join("mr")));
}

#[test]
fn a_symlink_to_a_non_executable_is_skipped() {
    let scratch = Scratch::new("symlink-non-executable");
    let (a, b, real) = (scratch.dir("a"), scratch.dir("b"), scratch.dir("real"));
    let cwd = scratch.dir("cwd");
    file(&real, "notes", 0o644);
    symlink(real.join("notes"), a.join("mr")).unwrap();
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(b.join("mr")));
}

#[test]
fn a_dangling_symlink_is_skipped() {
    let scratch = Scratch::new("dangling");
    let (a, cwd) = (scratch.dir("a"), scratch.dir("cwd"));
    symlink(scratch.0.join("gone"), a.join("mr")).unwrap();
    assert_eq!(first_on_path("mr", Some(path_of(&[a.to_str().unwrap()]).as_os_str()), &cwd), None);
}

#[test]
fn a_directory_with_the_name_is_skipped() {
    let scratch = Scratch::new("directory");
    let (a, b, cwd) = (scratch.dir("a"), scratch.dir("b"), scratch.dir("cwd"));
    fs::create_dir(a.join("mr")).unwrap();
    fs::set_permissions(a.join("mr"), fs::Permissions::from_mode(0o755)).unwrap();
    file(&b, "mr", 0o755);
    let path = path_of(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(b.join("mr")));
}

#[test]
fn an_executable_with_another_name_does_not_match() {
    let scratch = Scratch::new("other-name");
    let (a, cwd) = (scratch.dir("a"), scratch.dir("cwd"));
    file(&a, "mortar", 0o755);
    file(&a, "mrx", 0o755);
    assert_eq!(first_on_path("mr", Some(path_of(&[a.to_str().unwrap()]).as_os_str()), &cwd), None);
}

#[test]
fn a_dir_that_does_not_exist_is_skipped() {
    let scratch = Scratch::new("missing-dir");
    let (a, cwd) = (scratch.dir("a"), scratch.dir("cwd"));
    file(&a, "mr", 0o755);
    let missing = scratch.0.join("missing");
    let path = path_of(&[missing.to_str().unwrap(), a.to_str().unwrap()]);
    assert_eq!(first_on_path("mr", Some(path.as_os_str()), &cwd), Some(a.join("mr")));
}

#[test]
fn an_unset_path_gives_none_even_with_an_executable_in_cwd() {
    let scratch = Scratch::new("unset-path");
    let cwd = scratch.dir("cwd");
    file(&cwd, "mr", 0o755);
    assert_eq!(first_on_path("mr", None, &cwd), None);
}
