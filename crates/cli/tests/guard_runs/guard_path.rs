//! `mr guard install` joins CLI PATH/marker reads with Land's real guard installation,
//! Gitio and Oplog locks. Each Fixture owns/removes its repository and candidates. Only child PATH changes.
//! The 22-case inventory's directory/missing-path marker-reader cases cannot reach that reader through the binary:
//! PATH lookup rejects them first. The marked-symlink case proves recognition after canonicalization, not that
//! file_has_marker follows a symlink itself. Empty/unset PATH cases must install successfully, never skip.
use crate::support::{actual_git, script, Fixture};
use std::{ffi::{OsStr, OsString}, fs, os::unix::fs::{symlink, PermissionsExt}, path::{Path, PathBuf}};

// The binary's actual 37-byte signature is protocol input, not an import of CLI's private marker implementation.
const MARKER: &[u8] = b"\0mortar-binary-marker/5e3a9d7c1e4b9a\0";
const FOREIGN: &[u8] = b"#!/bin/sh\nprintf 'candidate ran\\n' >\"$GUARD_PATH_CANDIDATE_RAN\"\nexit 97\n";

enum Warning<'a> { Recognized, Missing, Foreign(&'a Path) }

// Keep git and the lock's real hostname program reachable without appending ambient PATH (and an ambient mr).
fn tools(dir: &Path) {
    symlink(actual_git(), dir.join("git")).unwrap();
    let hostname = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|entry| entry.join("hostname")).find(|p| p.is_file()).unwrap().canonicalize().unwrap();
    symlink(hostname, dir.join("hostname")).unwrap();
}

fn bin(f: &Fixture, name: &str) -> PathBuf {
    let dir = f.root.join(name);
    fs::create_dir(&dir).unwrap();
    tools(&dir);
    dir
}

fn path(entries: &[&Path]) -> OsString { std::env::join_paths(entries).unwrap() }

fn candidate(file: &Path, bytes: &[u8], executable: bool) {
    fs::write(file, bytes).unwrap();
    fs::set_permissions(file, fs::Permissions::from_mode(if executable { 0o755 } else { 0o644 })).unwrap();
}

fn foreign(dir: &Path, executable: bool) -> PathBuf {
    let file = dir.join("mr");
    candidate(&file, FOREIGN, executable);
    file
}

fn install(f: &Fixture, child_path: Option<&OsStr>, expected: Warning<'_>) {
    let program = Path::new(env!("CARGO_BIN_EXE_mortar")).canonicalize().unwrap();
    let before = f.observe();
    let sentinel = f.root.join("candidate-ran");
    let mut command = f.command(&f.request, &["--actor", "guard-path-harness", "guard", "install"]);
    command.env("GUARD_PATH_CANDIDATE_RAN", &sentinel);
    match child_path {
        Some(value) => { command.env("PATH", value); }
        None => { command.env_remove("PATH"); }
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let way_in = format!("run Mortar as mortar ({})", program.display());
    let warning = match expected {
        Warning::Recognized => String::new(),
        Warning::Missing => format!("no mr is on PATH: {way_in}\n"),
        Warning::Foreign(first) => format!("the first mr on PATH is {}, not this Mortar: {way_in}\n", first.display()),
    };
    assert_eq!(String::from_utf8(output.stdout).unwrap(), format!(
        "the guard is installed in {}\nit guards refs/heads/integration and refs/heads/main, and pushes of them to origin\n{}next: nothing to do: git now refuses moves of integration and main that Mortar did not make\n",
        f.common.join("hooks").display(), warning));
    assert_eq!(fs::read(f.common.join("mortar/guard.json")).unwrap(),
        b"{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}\n");
    for (name, bytes) in [
        ("reference-transaction", b"#!/bin/sh\nexec mortar guard-hook reference-transaction \"$@\"\n".as_slice()),
        ("pre-push", b"#!/bin/sh\nexec mortar guard-hook pre-push \"$@\"\n".as_slice()),
    ] {
        let hook = f.common.join("hooks").join(name);
        assert_eq!(fs::read(&hook).unwrap(), bytes);
        assert_ne!(fs::metadata(hook).unwrap().permissions().mode() & 0o111, 0);
    }
    assert_eq!(f.observe(), before, "guard installation must preserve refs, index, work and op log");
    assert!(!sentinel.exists(), "foreign PATH candidates must never run, even if their failure is ignored");
}

#[test]
fn a_file_holding_the_marker_has_it() {
    let f = Fixture::plain(); let a = bin(&f, "a");
    candidate(&a.join("mr"), &[b"head ".as_slice(), MARKER, b" tail"].concat(), true);
    install(&f, Some(a.as_os_str()), Warning::Recognized);
}

#[test]
fn a_file_without_the_marker_does_not_have_it() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let first = foreign(&a, true);
    install(&f, Some(a.as_os_str()), Warning::Foreign(&first));
}

#[test]
fn the_marker_straddling_the_64_kib_boundary_is_found() {
    let f = Fixture::plain(); let a = bin(&f, "a");
    candidate(&a.join("mr"), &[vec![b'x'; 65533], MARKER.to_vec(), vec![b'y'; 100]].concat(), true);
    install(&f, Some(a.as_os_str()), Warning::Recognized);
}

#[test]
fn a_file_shorter_than_one_read_with_the_marker_near_the_start_has_it() {
    let f = Fixture::plain(); let a = bin(&f, "a");
    candidate(&a.join("mr"), &[b"ab".as_slice(), MARKER].concat(), true);
    install(&f, Some(a.as_os_str()), Warning::Recognized);
}

#[test]
fn a_symlink_to_a_file_holding_the_marker_has_it() {
    // Partial inventory case 7: the binary canonicalizes the candidate before the marker reader receives its path.
    let f = Fixture::plain(); let a = bin(&f, "a"); let target = f.root.join("marked-target");
    candidate(&target, &[b"head ".as_slice(), MARKER, b" tail"].concat(), true);
    symlink(&target, a.join("mr")).unwrap();
    install(&f, Some(a.as_os_str()), Warning::Recognized);
}

#[test]
fn the_first_of_two_dirs_holding_an_executable_wins() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let b = bin(&f, "b");
    let first = foreign(&a, true); foreign(&b, true);
    install(&f, Some(&path(&[&a, &b])), Warning::Foreign(&first));
}

#[test]
fn a_non_executable_file_is_skipped() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let b = bin(&f, "b");
    foreign(&a, false); let first = foreign(&b, true);
    install(&f, Some(&path(&[&a, &b])), Warning::Foreign(&first));
}

#[test]
fn only_non_executables_give_none() {
    let f = Fixture::plain(); let a = bin(&f, "a"); foreign(&a, false);
    install(&f, Some(a.as_os_str()), Warning::Missing);
}

#[test]
fn empty_entries_without_the_name_in_cwd_fall_through_to_the_next_entry() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let first = foreign(&a, true);
    install(&f, Some(&path(&[Path::new(""), Path::new(""), &a, Path::new("")])), Warning::Foreign(&first));
}

#[test]
fn a_leading_empty_entry_finds_the_cwd_executable_before_a_later_dir() {
    let f = Fixture::plain(); let safe = bin(&f, "safe"); foreign(&safe, true);
    let first = foreign(&f.request, true);
    install(&f, Some(&path(&[Path::new(""), &safe])), Warning::Foreign(&first));
}

#[test]
fn a_trailing_empty_entry_finds_the_cwd_executable_when_no_dir_has_one() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let first = foreign(&f.request, true);
    install(&f, Some(&path(&[&a, Path::new("")])), Warning::Foreign(&first));
}

#[test]
fn a_relative_entry_is_looked_up_under_cwd() {
    let f = Fixture::plain(); let sub = f.request.join("sub"); tools(&sub); let first = foreign(&sub, true);
    install(&f, Some(OsStr::new("sub")), Warning::Foreign(&first));
}

#[test]
fn an_empty_path_with_no_executable_in_cwd_gives_none() {
    let f = Fixture::plain();
    // An empty PATH must still reach real Git and hostname for the actual guard install to complete.
    tools(&f.request);
    install(&f, Some(OsStr::new("")), Warning::Missing);
}

#[test]
fn a_symlink_to_an_executable_counts_and_is_named_by_its_path_entry() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let target = f.root.join("foreign-target");
    script(&target, FOREIGN); let first = a.join("mr"); symlink(&target, &first).unwrap();
    assert_ne!(first, first.canonicalize().unwrap());
    install(&f, Some(a.as_os_str()), Warning::Foreign(&first));
}

#[test]
fn a_symlink_to_a_non_executable_is_skipped() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let b = bin(&f, "b");
    let target = f.root.join("not-executable"); candidate(&target, FOREIGN, false);
    symlink(&target, a.join("mr")).unwrap(); let first = foreign(&b, true);
    install(&f, Some(&path(&[&a, &b])), Warning::Foreign(&first));
}

#[test]
fn a_dangling_symlink_is_skipped() {
    let f = Fixture::plain(); let a = bin(&f, "a");
    symlink(f.root.join("missing-target"), a.join("mr")).unwrap();
    install(&f, Some(a.as_os_str()), Warning::Missing);
}

#[test]
fn a_directory_with_the_name_is_skipped() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let b = bin(&f, "b");
    fs::create_dir(a.join("mr")).unwrap(); let first = foreign(&b, true);
    install(&f, Some(&path(&[&a, &b])), Warning::Foreign(&first));
}

#[test]
fn an_executable_with_another_name_does_not_match() {
    let f = Fixture::plain(); let a = bin(&f, "a");
    script(&a.join("mortar"), FOREIGN); script(&a.join("mrx"), FOREIGN);
    install(&f, Some(a.as_os_str()), Warning::Missing);
}

#[test]
fn a_dir_that_does_not_exist_is_skipped() {
    let f = Fixture::plain(); let a = bin(&f, "a"); let first = foreign(&a, true);
    install(&f, Some(&path(&[&f.root.join("missing-dir"), &a])), Warning::Foreign(&first));
}

#[test]
fn an_unset_path_gives_none_even_with_an_executable_in_cwd() {
    let f = Fixture::plain(); foreign(&f.request, true);
    // Unlike PATH="", an unset PATH must not search cwd for mr. Git/hostname use the platform's default lookup.
    install(&f, None, Warning::Missing);
}
