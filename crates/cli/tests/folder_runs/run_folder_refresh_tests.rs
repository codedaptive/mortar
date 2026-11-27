//! `mortar folder refresh <zone folder path>`: the zone folder's sparse paths set to the config's current zone paths
//! and root files, then the folder brought to its zone branch's tip by the zone-folder move; a repeat with nothing
//! new is already-refreshed; a request or attempt folder is refused with mr join as next; an unstaged edit in the
//! way and a staged change matching no zone commit refuse the move with the work kept; no path is a usage error
//! whose usage line names refresh; a plain directory and a plain git worktree are not-a-folder; a zone folder whose
//! zone the config no longer names is refused with mr init --replace next; a live join-core lock is busy. Each
//! refusal leaves the folder as it was.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it. The zone folder, the
//! request folder and the join are made through the binary, so the op log records every value zone/core held.

use std::fs;
use std::path::{Path, PathBuf};

use crate::json::{check, Report};
use crate::support::{raw, write, Fixture, ZoneConfig, ZONE};
use gitio::{Git, Oid};
use oplog::{KillPidProbe, LockName, LockRecord};

/// A folder's index entries, its files' bytes by path, and all refs of the repository.
type State = (Vec<u8>, Vec<(PathBuf, Vec<u8>)>, Vec<u8>);

/// A main repository whose config first gives zone core only `src`, a zone folder made at the base on that cone,
/// and a request folder core/task on `src`.
struct Refresh {
    f: Fixture,
    zone: PathBuf,
    request: PathBuf,
}

impl Refresh {
    fn new() -> Self {
        let f = Fixture::plain();
        let mut narrow = f.init_args.clone();
        narrow.zones.insert("core".into(), ZoneConfig { paths: vec!["src".into()] });
        f.configure(&narrow);
        let zone = f.root.join("zone-core");
        let request = f.root.join("request-core-task");
        let s = Self { f, zone, request };
        s.harness(&s.f.git.repo, &["folder", "zone", "core", s.zone.to_str().unwrap(), "--start", s.f.base.as_str()], 0, "made");
        s.harness(&s.f.git.repo, &["folder", "request", "core", "task", s.request.to_str().unwrap(), "--paths", "src"], 0, "made");
        s
    }

    /// `mortar --json --actor harness <words>` in `cwd`, checked for verb words[0], exit and outcome.
    fn harness(&self, cwd: &Path, words: &[&str], exit: i32, outcome: &str) -> Report {
        let mut command = self.f.command(cwd, &["--json", "--actor", "harness"]);
        check(command.args(words).output().unwrap(), words[0], exit, outcome)
    }

    fn refresh(&self, cwd: &Path, path: &str, exit: i32, outcome: &str) -> Report {
        self.harness(cwd, &["folder", "refresh", path], exit, outcome)
    }

    /// The edits written in the request folder and joined through `mr join`: the zone tip after it.
    fn join(&self, edits: &[(&str, &[u8])]) -> Oid {
        for (path, bytes) in edits { write(&self.request, path, bytes); }
        check(self.f.command(&self.request, &["--json", "join"]).output().unwrap(), "join", 0, "joined");
        self.f.git.read_ref(ZONE).unwrap().unwrap()
    }

    /// The config's zone core back to `src` and `sub`, through `mr init --replace`.
    fn widen(&self) { self.f.configure(&self.f.init_args); }

    fn git(&self, folder: &Path) -> Git { Git { repo: folder.into(), ..self.f.git.clone() } }

    /// Every index entry and every file's bytes under `folder` (its .git file left out), and all refs.
    fn state(&self, folder: &Path) -> State {
        fn files(root: &Path, at: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in fs::read_dir(at).unwrap() {
                let path = entry.unwrap().path();
                if path.file_name().unwrap() == ".git" { continue; }
                if path.is_dir() { files(root, &path, out); } else { out.push((path.strip_prefix(root).unwrap().into(), fs::read(path).unwrap())); }
            }
        }
        let mut bytes = Vec::new();
        files(folder, folder, &mut bytes);
        bytes.sort();
        (raw(&self.git(folder), &["ls-files", "--stage", "-z"]), bytes,
         raw(&self.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]))
    }
}

#[test]
fn refresh_widens_the_cone_and_brings_the_zone_folder_to_the_tip_then_a_repeat_changes_nothing() {
    let s = Refresh::new();
    let tip = s.join(&[("src/joined", b"joined work\n")]);
    s.widen();
    assert!(!s.zone.join("sub/keep").exists());
    assert!(!s.zone.join("src/joined").exists());
    // Relative to the cwd: the main checkout and the zone folder are siblings under the fixture's root.
    let r = s.refresh(&s.f.git.repo, "../zone-core", 0, "refreshed");
    assert_eq!(r.text("folder"), s.zone.display().to_string());
    assert_eq!(r.text("zone"), "core");
    assert_eq!(r.text("tip"), tip.as_str());
    let mut paths = r.strings("paths").to_vec();
    paths.sort();
    assert_eq!(paths, ["root.txt", "src", "sub"]);
    assert!(r.flag("paths_changed"));
    assert!(r.flag("moved"));
    assert_eq!(fs::read(s.zone.join("sub/keep")).unwrap(), b"keep\n");
    assert_eq!(fs::read(s.zone.join("src/joined")).unwrap(), b"joined work\n");
    assert_eq!(s.f.git.index_tree(&s.zone).unwrap(), s.f.git.commit_info(&tip).unwrap().tree);
    assert_eq!(raw(&s.git(&s.zone), &["status", "--porcelain"]), b"");
    let before = s.state(&s.zone);
    let again = s.refresh(&s.f.git.repo, s.zone.to_str().unwrap(), 0, "already-refreshed");
    assert!(!again.flag("paths_changed"));
    assert!(!again.flag("moved"));
    assert_eq!(again.text("tip"), tip.as_str());
    assert_eq!(s.state(&s.zone), before);
}

#[test]
fn refresh_of_a_request_folder_is_refused_with_mr_join_next() {
    let s = Refresh::new();
    let before = s.state(&s.request);
    let r = s.refresh(&s.f.git.repo, s.request.to_str().unwrap(), 3, "refused");
    assert!(r.text("next").contains("mr join"), "{}", r.text("next"));
    assert_eq!(s.state(&s.request), before);
}

#[test]
fn refresh_of_an_attempt_folder_is_refused_with_mr_join_next() {
    let s = Refresh::new();
    let tries = s.f.root.join("tries");
    s.harness(&s.request, &["folder", "attempts", "1", tries.to_str().unwrap()], 0, "made");
    let attempt = tries.join("try-1");
    let before = s.state(&attempt);
    let r = s.refresh(&s.f.git.repo, attempt.to_str().unwrap(), 3, "refused");
    assert!(r.text("next").contains("mr join"), "{}", r.text("next"));
    assert_eq!(s.state(&attempt), before);
}

#[test]
fn an_unstaged_edit_in_the_way_refuses_the_move_and_keeps_the_edit() {
    let s = Refresh::new();
    s.join(&[("src/file", b"joined work\n")]);
    write(&s.zone, "src/file", b"owner's unstaged edit\n");
    let base_tree = s.f.git.commit_info(&s.f.base).unwrap().tree;
    let zone = s.zone.to_str().unwrap();
    let r = s.refresh(&s.f.git.repo, zone, 3, "folder-not-moved");
    assert!(r.text("next").ends_with(&format!("mr folder refresh {zone} --actor <name>")), "{}", r.text("next"));
    assert_eq!(fs::read(s.zone.join("src/file")).unwrap(), b"owner's unstaged edit\n");
    assert_eq!(s.f.git.index_tree(&s.zone).unwrap(), base_tree);
}

#[test]
fn a_staged_change_matching_no_zone_commit_refuses_the_move_and_keeps_the_staged_edit() {
    let s = Refresh::new();
    s.join(&[("src/joined", b"joined work\n")]);
    write(&s.zone, "src/staged", b"owner's staged edit\n");
    raw(&s.git(&s.zone), &["add", "src/staged"]);
    let before = s.state(&s.zone);
    s.refresh(&s.f.git.repo, s.zone.to_str().unwrap(), 3, "folder-not-moved");
    assert_eq!(raw(&s.git(&s.zone), &["show", ":src/staged"]), b"owner's staged edit\n");
    assert_eq!(fs::read(s.zone.join("src/staged")).unwrap(), b"owner's staged edit\n");
    assert!(!s.zone.join("src/joined").exists());
    assert_eq!(s.state(&s.zone).0, before.0);
}

#[test]
fn refresh_with_no_path_is_a_usage_error_naming_refresh() {
    let s = Refresh::new();
    let before = s.state(&s.zone);
    let r = s.harness(&s.f.git.repo, &["folder", "refresh"], 2, "usage");
    assert!(r.text("next").contains("refresh <"), "{}", r.text("next"));
    assert_eq!(s.state(&s.zone), before);
}

#[test]
fn refresh_of_a_path_that_is_not_a_mortar_folder_is_not_a_folder() {
    let s = Refresh::new();
    let plain_dir = s.f.root.join("plain-dir");
    write(&plain_dir, "file", b"plain\n");
    let before = s.state(&s.zone);
    s.refresh(&s.f.git.repo, plain_dir.to_str().unwrap(), 3, "not-a-folder");
    assert_eq!(fs::read(plain_dir.join("file")).unwrap(), b"plain\n");
    assert_eq!(s.state(&s.zone), before);
    // A git worktree Mortar did not make: git finds it, but it holds no folder record.
    let worktree = s.f.root.join("plain-worktree");
    raw(&s.f.git, &["worktree", "add", "-q", "--detach", worktree.to_str().unwrap(), s.f.base.as_str()]);
    let before = s.state(&worktree);
    s.refresh(&s.f.git.repo, worktree.to_str().unwrap(), 3, "not-a-folder");
    assert_eq!(s.state(&worktree), before);
}

#[test]
fn refresh_of_a_zone_folder_whose_zone_the_config_no_longer_names_is_refused_with_mr_init_replace_next() {
    let s = Refresh::new();
    let mut other = s.f.init_args.clone();
    other.zones.remove("core");
    other.zones.insert("other".into(), ZoneConfig { paths: vec!["sub".into()] });
    s.f.configure(&other);
    let before = s.state(&s.zone);
    let r = s.refresh(&s.f.git.repo, s.zone.to_str().unwrap(), 3, "refused");
    let next = r.text("next");
    assert!(next.starts_with("mr init --replace"), "{next}");
    assert!(next.contains("--zone core="), "{next}");
    assert_eq!(s.state(&s.zone), before);
}

#[test]
fn refresh_with_the_join_lock_held_by_a_live_pid_is_busy_and_changes_nothing() {
    let s = Refresh::new();
    s.join(&[("src/joined", b"joined work\n")]);
    s.widen();
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "refresh-live-holder".into(), since: 456 };
    let held = oplog::acquire_lock(&s.f.common, &LockName::Join("core".into()), &holder, &KillPidProbe).unwrap();
    let before = s.state(&s.zone);
    let r = s.refresh(&s.f.git.repo, s.zone.to_str().unwrap(), 4, "busy");
    assert_eq!(r.number("holder_pid"), u64::from(holder.pid));
    assert_eq!(r.text("holder_key"), "refresh-live-holder");
    assert!(!s.zone.join("sub/keep").exists());
    assert!(!s.zone.join("src/joined").exists());
    assert_eq!(s.state(&s.zone), before);
    oplog::release_lock(&held).unwrap();
}
