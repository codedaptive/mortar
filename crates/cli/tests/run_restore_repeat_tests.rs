//! CLI restore's crash-repeat and unsaved-work outcomes. This is a ZIT (S-119); the crash-repeat case is red-first.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::{Config, ZoneConfig};
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
        cli::write_config(&common, &Config {
            origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
            zones: BTreeMap::from([("core".into(), ZoneConfig { paths })]), root_files: vec![],
            build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP, floor_bytes: 0, formatter: None,
        }).unwrap();
        fs::write(request.join("src/file"), b"checkpoint\n").unwrap();
        let output = command(&repo.path, &request, &git).args(["save", "Checkpoint"]).output().unwrap();
        assert!(output.status.success(), "checkpoint save: {output:?}");
        let checkpoint = oplog::read_op_log(&common).unwrap().records.last().unwrap().op_id.clone();
        let branch_at_checkpoint = git.read_ref(REQUEST).unwrap().unwrap();
        let snap_at_checkpoint = git.read_ref(SNAP).unwrap().unwrap();
        fs::write(request.join("src/file"), b"saved later\n").unwrap();
        let output = command(&repo.path, &request, &git).args(["save", "Later work"]).output().unwrap();
        assert!(output.status.success(), "later save: {output:?}");
        let branch_saved = git.read_ref(REQUEST).unwrap().unwrap();
        let snap_saved = git.read_ref(SNAP).unwrap().unwrap();
        assert_ne!(branch_at_checkpoint, branch_saved);
        assert_ne!(snap_at_checkpoint, snap_saved);
        Self { repo, git, common, request, checkpoint, branch_at_checkpoint, snap_at_checkpoint, branch_saved, snap_saved }
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
    let shim = fx.repo.path.join("git-crash-boundary.sh");
    let shimbin = fx.repo.path.join("shimbin");
    fs::create_dir(&shimbin).unwrap();
    write_git_kill_shim(&shim).unwrap();
    let original_path = std::env::var("PATH").unwrap();
    let script = format!("#!/bin/sh\nPATH='{}' exec '{}' \"$@\"\n", original_path, shim.display());
    write_git_shim(&shimbin.join("git"), script.as_bytes()).unwrap();
    let mut worker = test_child("restore_worker").unwrap();
    worker.env("MORTAR_TEST_ROOT", &fx.repo.path).env("MORTAR_TEST_REQUEST", &fx.request)
        .env("MORTAR_TEST_CHECKPOINT", &fx.checkpoint).env("MORTAR_TEST_SHIMBIN", &shimbin)
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
    for name in ["TESTKIT_GATE_RELEASE", "TESTKIT_GATE_ACK", "TESTKIT_GATE_ARMED"] {
        mortar.env(name, std::env::var_os(name).unwrap());
    }
    panic!("exec mortar: {}", mortar.exec());
}
