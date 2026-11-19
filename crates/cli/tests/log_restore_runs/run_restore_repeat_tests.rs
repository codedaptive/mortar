//! CLI restore's crash-repeat, unsaved-work and repeat-after-a-later-restore outcomes, and a crash-repeat refusing
//! work added after the kill.
//! The mortar binary runs as a child in a throwaway repository (testkit's TestRepo) with a zone
//! folder, a request folder and the config `mr init` writes (zone core=src, floor 0, the default build dir),
//! reaching folders, snapshot, gitio and oplog below cli; the cli lane owns it. The crash cases kill
//! the worker (`run_restore_repeat_tests::restore_worker`, run by testkit's test_child) at testkit's git crash gate.

use std::fs;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::{Git, Oid, RunOpts};
use oplog::{KillPidProbe, OpPhase};
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim, write_git_shim, TestRepo};

const REQUEST: &str = "refs/heads/req/core/task";
const SNAP: &str = "refs/mortar/snap/core/task";

struct Fixture {
    repo: TestRepo,
    git: Git,
    common: PathBuf,
    request: PathBuf,
    checkpoint: String,
    later: String,
    branch_at_checkpoint: Oid,
    snap_at_checkpoint: Oid,
    branch_saved: Oid,
    snap_saved: Oid,
}

impl Fixture {
    fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_AUTHOR_NAME".into(), "Test Author".into()),
            ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
            ("GIT_COMMITTER_NAME".into(), "Test Committer".into()),
            ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ];
        fs::create_dir_all(repo.path.join("src")).unwrap();
        fs::write(repo.path.join("src/file"), b"base\n").unwrap();
        git.run(&["add", "-A"], &RunOpts::default()).unwrap();
        git.run(&["commit", "-qm", "base"], &RunOpts::default()).unwrap();
        let base = git.rev_parse("HEAD").unwrap().unwrap();
        let common = git.common_dir().unwrap();
        let zone = repo.path.join("zone-folder");
        let request = repo.path.join("request-folder");
        let paths = vec!["src".to_string()];
        let setup = |path: &Path| FolderSetup {
            git: &git, path: path.into(), floor_bytes: 0,
            sweep: SweepEstimate { bytes: 0, reason: None }, disk: &DfFreeDisk,
            pids: &KillPidProbe, actor: "fixture".into(),
        };
        oplog::recover(&git, &common).unwrap();
        folders::make_zone_folder(&setup(&zone), "core", &paths, Some(&base)).unwrap();
        oplog::recover(&git, &common).unwrap();
        folders::make_request_folder(&setup(&request), "core", "task", &paths).unwrap();
        let output = command(&repo.path, &repo.path, &git).args([
            "--actor", "harness", "init", "--origin", "main", "--integration", "integration", "--remote", "origin",
            "--zone", "core=src", "--floor-bytes", "0",
        ]).output().unwrap();
        assert!(output.status.success(), "init: {output:?}");
        fs::write(request.join("src/file"), b"checkpoint\n").unwrap();
        let output = command(&repo.path, &request, &git).args(["save", "Checkpoint"]).output().unwrap();
        assert!(output.status.success(), "checkpoint save: {output:?}");
        let checkpoint = oplog::read_op_log(&common).unwrap().records.last().unwrap().op_id.clone();
        let branch_at_checkpoint = git.read_ref(REQUEST).unwrap().unwrap();
        let snap_at_checkpoint = git.read_ref(SNAP).unwrap().unwrap();
        fs::write(request.join("src/file"), b"saved later\n").unwrap();
        let output = command(&repo.path, &request, &git).args(["save", "Later work"]).output().unwrap();
        assert!(output.status.success(), "later save: {output:?}");
        let later = oplog::read_op_log(&common).unwrap().records.last().unwrap().op_id.clone();
        let branch_saved = git.read_ref(REQUEST).unwrap().unwrap();
        let snap_saved = git.read_ref(SNAP).unwrap().unwrap();
        assert_ne!(branch_at_checkpoint, branch_saved);
        assert_ne!(snap_at_checkpoint, snap_saved);
        Self { repo, git, common, request, checkpoint, later, branch_at_checkpoint, snap_at_checkpoint, branch_saved, snap_saved }
    }

    fn restore(&self) -> Output {
        command(&self.repo.path, &self.request, &self.git)
            .args(["--actor", "harness", "restore", &self.checkpoint, REQUEST, SNAP]).output().unwrap()
    }
}

fn command(root: &Path, cwd: &Path, git: &Git) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mortar"));
    command.current_dir(cwd).env_clear().envs(git.env.iter().cloned())
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "restore-test").stdin(Stdio::null());
    command
}

#[test]
fn repeat_after_ref_transaction_finishes_the_folder_move_once() {
    let fx = Fixture::new();
    let shim = fx.repo.path.join("git-crash'boundary.sh");
    let shimbin = fx.repo.path.join("shimbin");
    fs::create_dir(&shimbin).unwrap();
    write_git_kill_shim(&shim).unwrap();
    // The wrapper's text holds no paths, so a quote in PATH, TMPDIR or the shim's name cannot break it; the
    // values come by environment. Its PATH leaves out shimbin, so the kill shim's own `git` is the real one.
    write_git_shim(&shimbin.join("git"), b"#!/bin/sh\nPATH=\"$MORTAR_TEST_GIT_PATH\" exec \"$MORTAR_TEST_GIT_SHIM\" \"$@\"\n").unwrap();
    let mut worker = test_child("run_restore_repeat_tests::restore_worker").unwrap();
    worker.env("MORTAR_TEST_ROOT", &fx.repo.path).env("MORTAR_TEST_REQUEST", &fx.request)
        .env("MORTAR_TEST_CHECKPOINT", &fx.checkpoint).env("MORTAR_TEST_SHIMBIN", &shimbin)
        .env("MORTAR_TEST_GIT_PATH", std::env::var_os("PATH").unwrap()).env("MORTAR_TEST_GIT_SHIM", &shim)
        .env("MORTAR_TEST_CRASH", "after");
    let killed = run_with_kill_gate(&mut worker, &fx.repo.path).unwrap();
    assert_eq!(killed.status.signal(), Some(9), "worker: {killed:?}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_at_checkpoint.clone()));
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), Some(fx.snap_at_checkpoint.clone()));
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"saved later\n");
    let records = oplog::read_op_log(&fx.common).unwrap().records;
    let intent = records.iter().find(|r| r.kind == "restore" && r.phase == OpPhase::Intent).unwrap();
    let key = intent.key.clone();
    assert!(!records.iter().any(|r| r.key == key && r.phase == OpPhase::Done));
    let repeated = fx.restore();
    assert_eq!(repeated.status.code(), Some(0), "repeat stdout: {}\nstderr: {}", String::from_utf8_lossy(&repeated.stdout), String::from_utf8_lossy(&repeated.stderr));
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_at_checkpoint.clone()));
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"checkpoint\n");
    let records = oplog::read_op_log(&fx.common).unwrap().records;
    assert_eq!(records.iter().filter(|r| r.kind == "restore" && r.key == key && r.phase == OpPhase::Done).count(), 1);
}

/// A crash-repeat whose folder gained work after the kill refuses and moves nothing: before the folder move,
/// the repeat makes the saved check against the snapshot ref's old value, finds the new file unsaved, exits 1
/// with the unsaved-work report, and leaves the refs, the folder and the folder-move count as they were.
#[test]
fn crash_repeat_with_work_added_after_the_kill_refuses_and_moves_nothing() {
    let fx = Fixture::new();
    let shim = fx.repo.path.join("git-crash'boundary.sh");
    let shimbin = fx.repo.path.join("shimbin");
    fs::create_dir(&shimbin).unwrap();
    write_git_kill_shim(&shim).unwrap();
    write_git_shim(&shimbin.join("git"), b"#!/bin/sh\nPATH=\"$MORTAR_TEST_GIT_PATH\" exec \"$MORTAR_TEST_GIT_SHIM\" \"$@\"\n").unwrap();
    let mut worker = test_child("run_restore_repeat_tests::restore_worker").unwrap();
    worker.env("MORTAR_TEST_ROOT", &fx.repo.path).env("MORTAR_TEST_REQUEST", &fx.request)
        .env("MORTAR_TEST_CHECKPOINT", &fx.checkpoint).env("MORTAR_TEST_SHIMBIN", &shimbin)
        .env("MORTAR_TEST_GIT_PATH", std::env::var_os("PATH").unwrap()).env("MORTAR_TEST_GIT_SHIM", &shim)
        .env("MORTAR_TEST_CRASH", "after");
    let killed = run_with_kill_gate(&mut worker, &fx.repo.path).unwrap();
    assert_eq!(killed.status.signal(), Some(9), "worker: {killed:?}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_at_checkpoint.clone()));
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), Some(fx.snap_at_checkpoint.clone()));
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"saved later\n");
    let records = oplog::read_op_log(&fx.common).unwrap().records;
    let intent = records.iter().find(|r| r.kind == "restore" && r.phase == OpPhase::Intent).unwrap();
    let key = intent.key.clone();
    assert!(!records.iter().any(|r| r.key == key && r.phase == OpPhase::Done));
    // A new file, not an edit to src/file: git's own folder move refuses to overwrite an edited tracked file, but
    // carries an untracked one over silently, so only the saved check can stop this repeat.
    fs::write(fx.request.join("src/added"), b"added after the kill\n").unwrap();
    let folder_moves = || oplog::read_op_log(&fx.common).unwrap().records.iter().filter(|r| r.kind == "folder-move").count();
    let moves_before = folder_moves();
    let repeated = fx.restore();
    let out = format!("{}{}", String::from_utf8_lossy(&repeated.stdout), String::from_utf8_lossy(&repeated.stderr));
    assert_eq!(repeated.status.code(), Some(1), "repeat: {out}");
    let path = fx.request.display().to_string();
    assert!(out.lines().any(|l| l.contains(&path) && l.contains("holds work that is not saved")), "repeat: {out}");
    assert!(out.contains("nothing was moved"), "repeat: {out}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_at_checkpoint.clone()), "repeat: {out}");
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), Some(fx.snap_at_checkpoint.clone()), "repeat: {out}");
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"saved later\n", "repeat: {out}");
    assert_eq!(fs::read(fx.request.join("src/added")).unwrap(), b"added after the kill\n", "repeat: {out}");
    assert_eq!(folder_moves(), moves_before, "repeat: {out}");
    // The refusal returns before oplog::restore runs, so this Done can only be recover's finish of the killed
    // intent: the repeat ran as a repeat, not as a first restore.
    let records = oplog::read_op_log(&fx.common).unwrap().records;
    assert_eq!(records.iter().filter(|r| r.kind == "restore" && r.key == key && r.phase == OpPhase::Done).count(), 1, "repeat: {out}");
}

/// A finished restore repeated after a later restore moved the same ref on moves nothing: the folder and
/// its refs stay at the later restore's value, no folder move is recorded, and the output says why.
#[test]
fn repeat_after_a_later_restore_leaves_the_folder_and_its_ref() {
    let fx = Fixture::new();
    let text = |o: &Output| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    let first = fx.restore();
    assert_eq!(first.status.code(), Some(0), "R1: {}", text(&first));
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_at_checkpoint.clone()));
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"checkpoint\n");
    let later = command(&fx.repo.path, &fx.request, &fx.git)
        .args(["--actor", "harness", "restore", &fx.later, REQUEST, SNAP]).output().unwrap();
    assert_eq!(later.status.code(), Some(0), "R2: {}", text(&later));
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_saved.clone()));
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), Some(fx.snap_saved.clone()));
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"saved later\n");
    let folder_moves = || oplog::read_op_log(&fx.common).unwrap().records.iter().filter(|r| r.kind == "folder-move").count();
    let moves_before = folder_moves();
    let repeated = fx.restore();
    let out = text(&repeated);
    assert_eq!(repeated.status.code(), Some(0), "repeat: {out}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_saved.clone()), "repeat: {out}");
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), Some(fx.snap_saved.clone()), "repeat: {out}");
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"saved later\n", "repeat: {out}");
    assert_eq!(folder_moves(), moves_before, "repeat: {out}");
    let path = fx.request.display().to_string();
    assert!(out.lines().any(|l| l.contains(&path) && l.contains("stays") && l.contains("moved on")), "repeat: {out}");
}

#[test]
fn first_restore_refuses_unsaved_work_without_moving_refs() {
    let fx = Fixture::new();
    fs::write(fx.request.join("src/file"), b"unsaved edit\n").unwrap();
    let output = fx.restore();
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.lines().any(|line| line.contains(&fx.request.display().to_string()) && line.contains("holds work that is not saved")), "{text}");
    assert!(text.contains("nothing was moved"), "{text}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.branch_saved));
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), Some(fx.snap_saved));
    assert_eq!(fs::read(fx.request.join("src/file")).unwrap(), b"unsaved edit\n");
}

#[test]
#[ignore = "worker execs mortar under the crash gate"]
fn restore_worker() {
    let root = PathBuf::from(std::env::var_os("MORTAR_TEST_ROOT").unwrap());
    let request = PathBuf::from(std::env::var_os("MORTAR_TEST_REQUEST").unwrap());
    let checkpoint = std::env::var("MORTAR_TEST_CHECKPOINT").unwrap();
    let shimbin = PathBuf::from(std::env::var_os("MORTAR_TEST_SHIMBIN").unwrap());
    let path = std::env::join_paths(std::iter::once(shimbin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap()))).unwrap();
    let mut git = Git::new(&root);
    git.env = vec![("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into())];
    let mut mortar = command(&root, &request, &git);
    mortar.args(["--actor", "harness", "restore", &checkpoint, REQUEST, SNAP]).env("PATH", path)
        .env("MORTAR_TEST_WORKER_PID", std::process::id().to_string())
        .env("MORTAR_TEST_CRASH", std::env::var_os("MORTAR_TEST_CRASH").unwrap());
    for name in ["MORTAR_TEST_GIT_PATH", "MORTAR_TEST_GIT_SHIM", "TESTKIT_GATE_RELEASE", "TESTKIT_GATE_ACK", "TESTKIT_GATE_ARMED"] {
        mortar.env(name, std::env::var_os(name).unwrap());
    }
    panic!("exec mortar: {}", mortar.exec());
}
