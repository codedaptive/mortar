//! `mortar folder keep <k>`: the request branch and folder moved to the kept attempt, the attempts gone and their
//! snapshots kept; an unsaved attempt or request folder refused with nothing changed; an orphan attempt re-made in the
//! same command; and a keep killed between its op and the request folder's move finished by a plain repeat.
//! RIT (S-135, S-136): drives the mortar binary against real git and the crates below cli; owned by Nagatha from
//! landing.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-104, S-109):
//! - a request branch moved on after the attempts were cut (RequestMovedOn) and a try branch moved under the op
//!   (AttemptRefsLeft): the cli's why and next, never folders' FolderError Display text;
//! - an orphan with no trysnap ref (OrphanWithoutSnapshot): exit 1 naming the folder, nothing changed;
//! - a file the snapshot cap refuses in an attempt counts as unsaved (cli with snapshot's saved_state);
//! - a join-<zone> lock held by a live pid: exit 3 naming the holder's pid, host and since (cli with oplog's locks).

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::{Config, ZoneConfig};
use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::{Git, Oid, RunOpts};
use oplog::{KillPidProbe, OpRecord};
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim, write_git_shim, TestRepo};

const ACTOR: &str = "harness";
const REQUEST: &str = "refs/heads/req/core/task";
const TRY_PREFIX: &str = "refs/heads/try/core/task/";
const TRYSNAP_PREFIX: &str = "refs/mortar/trysnap/core/task/";

struct Fixture {
    repo: TestRepo,
    git: Git,
    common: PathBuf,
    request: PathBuf,
    dir: PathBuf,
    request_tip: Oid,
    tips: Vec<Oid>,
    trysnaps: Vec<Oid>,
}

impl Fixture {
    /// zone/core at a commit holding Cargo.toml, src/a/a.txt ("a") and src/b/b.txt; request folder core/task on src/a
    /// and Cargo.toml, saved with nothing changed; attempts 1 and 2 at <repo>/tries, attempt k with src/a/a.txt
    /// changed to "one" or "two" and saved by `mortar save` in its folder (a commit on its try branch and its trysnap
    /// ref).
    fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let git = fixture_git(&repo.path);
        fs::create_dir_all(repo.path.join("src/a")).unwrap();
        fs::create_dir_all(repo.path.join("src/b")).unwrap();
        fs::write(repo.path.join("Cargo.toml"), b"[workspace]\n").unwrap();
        fs::write(repo.path.join("src/a/a.txt"), b"a\n").unwrap();
        fs::write(repo.path.join("src/b/b.txt"), b"b\n").unwrap();
        git.run(&["add", "-A"], &RunOpts::default()).unwrap();
        git.run(&["commit", "-qm", "base"], &RunOpts::default()).unwrap();
        let base = git.rev_parse("HEAD").unwrap().unwrap();
        git.run(&["branch", "zone/core", base.as_str()], &RunOpts::default()).unwrap();
        let common = git.common_dir().unwrap();
        cli::write_config(&common, &Config {
            origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
            zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src".into()] })]),
            root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
            floor_bytes: 0, formatter: None,
        }).unwrap();
        let request = repo.path.join("request-folder");
        let dir = repo.path.join("tries");
        let paths: Vec<String> = vec!["src/a".into(), "Cargo.toml".into()];
        oplog::recover(&git, &common).unwrap();
        folders::make_request_folder(&setup(&git, &request), "core", "task", &paths).unwrap();
        save(&repo.path, &request, &git, "Start the task");
        let request_tip = git.read_ref(REQUEST).unwrap().unwrap();
        oplog::recover(&git, &common).unwrap();
        folders::make_attempts(&setup(&git, &dir), "core", "task", &paths, 2).unwrap();
        let mut tips = Vec::new();
        let mut trysnaps = Vec::new();
        for (k, content) in [(1, "one\n"), (2, "two\n")] {
            let folder = dir.join(format!("try-{k}"));
            fs::write(folder.join("src/a/a.txt"), content).unwrap();
            save(&repo.path, &folder, &git, &format!("Attempt {k}"));
            let tip = git.read_ref(&format!("{TRY_PREFIX}{k}")).unwrap().unwrap();
            assert_ne!(tip, request_tip, "attempt {k}'s save commits on its try branch");
            tips.push(tip);
            trysnaps.push(git.read_ref(&format!("{TRYSNAP_PREFIX}{k}")).unwrap().unwrap());
        }
        Self { repo, git, common, request, dir, request_tip, tips, trysnaps }
    }

    /// `mortar --actor harness --json folder keep <k>`, run in the request folder.
    fn keep(&self, k: u32) -> Output {
        command(&self.repo.path, &self.request, &self.git)
            .args(["--actor", ACTOR, "--json", "folder", "keep", &k.to_string()]).output().unwrap()
    }

    fn try_folder(&self, k: usize) -> PathBuf {
        self.dir.join(format!("try-{k}"))
    }

    /// The kept attempt 2 is in place: the request branch at its tip and the request folder holding its file, every
    /// attempt folder and try branch gone, and both trysnap refs where the attempts' saves left them.
    fn assert_kept_two(&self, out: &str) {
        assert_eq!(self.git.read_ref(REQUEST).unwrap(), Some(self.tips[1].clone()), "{out}");
        assert_eq!(fs::read(self.request.join("src/a/a.txt")).unwrap(), b"two\n", "{out}");
        assert!(!self.try_folder(1).exists(), "{out}");
        assert!(!self.try_folder(2).exists(), "{out}");
        assert!(self.git.list_refs(TRY_PREFIX).unwrap().is_empty(), "{out}");
        for k in 1..=2 {
            let trysnap = self.git.read_ref(&format!("{TRYSNAP_PREFIX}{k}")).unwrap();
            assert_eq!(trysnap, Some(self.trysnaps[k - 1].clone()), "trysnap {k}: {out}");
        }
    }

    /// Nothing of the fixture moved: the request branch and folder, both attempts' folders and branches, and no op
    /// logged since `before`.
    fn assert_nothing_changed(&self, before: usize, out: &str) {
        assert_eq!(self.git.read_ref(REQUEST).unwrap(), Some(self.request_tip.clone()), "{out}");
        for k in 1..=2 {
            assert!(self.try_folder(k).exists(), "try-{k}: {out}");
            let tip = self.git.read_ref(&format!("{TRY_PREFIX}{k}")).unwrap();
            assert_eq!(tip, Some(self.tips[k - 1].clone()), "try {k}: {out}");
        }
        assert_eq!(ops(&self.common).len(), before, "{out}");
    }
}

fn save(root: &Path, folder: &Path, git: &Git, sentence: &str) {
    let saved = command(root, folder, git).args(["save", sentence]).output().unwrap();
    assert!(saved.status.success(), "fixture save in {}: {}", folder.display(), text(&saved));
}

fn setup<'a>(git: &'a Git, path: &Path) -> FolderSetup<'a> {
    FolderSetup {
        git, path: path.into(), floor_bytes: 0, sweep: SweepEstimate { bytes: 0, reason: None },
        disk: &DfFreeDisk, pids: &KillPidProbe, actor: "fixture".into(),
    }
}

fn fixture_git(path: &Path) -> Git {
    let mut git = Git::new(path);
    git.env = vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ("GIT_AUTHOR_NAME".into(), "Test Author".into()),
        ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
        ("GIT_COMMITTER_NAME".into(), "Test Committer".into()),
        ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
    ];
    git
}

fn command(root: &Path, cwd: &Path, git: &Git) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mortar"));
    command.current_dir(cwd).env_clear().envs(git.env.iter().cloned())
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "folder-keep-test").stdin(Stdio::null());
    command
}

fn text(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

/// The string member `key` of the one JSON report on stdout; outcome and next come before any member a verb adds.
fn member(output: &Output, key: &str) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let tag = format!("\"{key}\":\"");
    let at = stdout.find(&tag).unwrap_or_else(|| panic!("no {key} in: {stdout}")) + tag.len();
    oplog::parse_json_string(&stdout, at).unwrap().0
}

/// A harness verb's next step is never aimed at an agent (SPEC-cli 05): it names no agent verb and asks no agent to
/// run anything.
fn assert_harness_next(next: &str) {
    for agent_verb in ["mortar save", "mortar join", "mortar need", "mortar resolve", "mortar status"] {
        assert!(!next.contains(agent_verb), "next names an agent verb: {next}");
    }
    assert!(!next.contains("agent run"), "next is aimed at an agent: {next}");
}

fn ops(common: &Path) -> Vec<OpRecord> {
    oplog::read_op_log(common).unwrap().records
}

/// Every op of the verb names the --actor. A stale-lock takeover is oplog's own record of taking the lock, with the
/// taker's pid@host as its actor (oplog takeover_op), not an op of the verb, so it is left out.
fn assert_ops_by_actor(records: &[OpRecord], out: &str) {
    for record in records.iter().filter(|r| r.kind != "lock-takeover") {
        assert_eq!(record.actor, ACTOR, "op {} ({}) logged by another actor: {out}", record.op_id, record.kind);
    }
}

#[test]
fn keep_of_a_saved_attempt_moves_the_request_to_it_and_removes_the_attempts() {
    let fx = Fixture::new();
    let before = ops(&fx.common).len();
    let kept = fx.keep(2);
    let out = text(&kept);
    assert_eq!(kept.status.code(), Some(0), "{out}");
    assert_eq!(member(&kept, "outcome"), "kept", "{out}");
    assert_harness_next(&member(&kept, "next"));
    fx.assert_kept_two(&out);
    let all = ops(&fx.common);
    let logged = &all[before..];
    assert!(logged.iter().any(|r| r.kind == "folder-keep"), "no folder-keep op: {logged:?}\n{out}");
    assert_ops_by_actor(logged, &out);
}

#[test]
fn an_unsaved_attempt_is_refused_and_nothing_changes() {
    let fx = Fixture::new();
    fs::write(fx.try_folder(1).join("src/a/a.txt"), b"one, unsaved\n").unwrap();
    let before = ops(&fx.common).len();
    let refused = fx.keep(2);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert_eq!(next, "repeat once each listed attempt's work is saved", "{out}");
    assert_harness_next(&next);
    fx.assert_nothing_changed(before, &out);
    assert_eq!(fs::read(fx.try_folder(1).join("src/a/a.txt")).unwrap(), b"one, unsaved\n", "{out}");
    assert_eq!(fs::read(fx.request.join("src/a/a.txt")).unwrap(), b"a\n", "{out}");
}

#[test]
fn a_request_folder_with_unsaved_work_is_refused_and_nothing_runs() {
    let fx = Fixture::new();
    fs::write(fx.request.join("src/a/a.txt"), b"request, unsaved\n").unwrap();
    let before = ops(&fx.common).len();
    let refused = fx.keep(2);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert_eq!(next, "repeat once the request folder's work is saved", "{out}");
    assert_harness_next(&next);
    fx.assert_nothing_changed(before, &out);
    assert_eq!(fs::read(fx.request.join("src/a/a.txt")).unwrap(), b"request, unsaved\n", "{out}");
}

#[test]
fn an_orphan_attempt_is_remade_and_the_keep_of_another_finishes_in_one_command() {
    let fx = Fixture::new();
    // Attempt 1 loses its try branch; its folder and trysnap ref stay, so it is an orphan remake_orphans can re-make.
    let try_one = format!("{TRY_PREFIX}1");
    fx.git.run(&["update-ref", "-d", try_one.as_str()], &RunOpts::default()).unwrap();
    let before = ops(&fx.common).len();
    let kept = fx.keep(2);
    let out = text(&kept);
    assert_eq!(kept.status.code(), Some(0), "{out}");
    assert_eq!(member(&kept, "outcome"), "kept", "{out}");
    assert_harness_next(&member(&kept, "next"));
    fx.assert_kept_two(&out);
    let all = ops(&fx.common);
    let logged = &all[before..];
    assert!(
        logged.iter().any(|r| r.kind == "folder-branch" && r.moves.iter().any(|m| m.refname == try_one)),
        "no folder-branch op re-made try 1: {logged:?}\n{out}"
    );
    assert!(logged.iter().any(|r| r.kind == "folder-keep"), "no folder-keep op: {logged:?}\n{out}");
    assert_ops_by_actor(logged, &out);
}

/// The kill shim fires at mortar's first `git update-ref`, which in a keep with every folder saved and no orphan is
/// keep's own folder-keep op (oplog apply is the only caller of update-ref); with MORTAR_TEST_CRASH=after the refs
/// move and the process dies before the request folder's move. A plain repeat finds the moved request branch and
/// finishes the folder move.
#[test]
fn a_keep_killed_between_its_op_and_the_folder_move_is_finished_by_a_plain_repeat() {
    let fx = Fixture::new();
    let shim = fx.repo.path.join("git-kill'boundary.sh");
    let shimbin = fx.repo.path.join("shimbin");
    fs::create_dir(&shimbin).unwrap();
    write_git_kill_shim(&shim).unwrap();
    // The wrapper's text holds no paths, so a quote in PATH, TMPDIR or the shim's name cannot break it; the values
    // come by environment. Its PATH leaves out shimbin, so the kill shim's own `git` is the real one.
    write_git_shim(&shimbin.join("git"), b"#!/bin/sh\nPATH=\"$MORTAR_TEST_GIT_PATH\" exec \"$MORTAR_TEST_GIT_SHIM\" \"$@\"\n").unwrap();
    let before = ops(&fx.common).len();
    let mut worker = test_child("run_folder_keep_tests::keep_worker").unwrap();
    worker.env("MORTAR_TEST_ROOT", &fx.repo.path).env("MORTAR_TEST_REQUEST", &fx.request)
        .env("MORTAR_TEST_SHIMBIN", &shimbin).env("MORTAR_TEST_GIT_PATH", std::env::var_os("PATH").unwrap())
        .env("MORTAR_TEST_GIT_SHIM", &shim).env("MORTAR_TEST_CRASH", "after");
    let killed = run_with_kill_gate(&mut worker, &fx.repo.path).unwrap();
    let out = text(&killed);
    assert_eq!(killed.status.signal(), Some(9), "worker: {killed:?}");
    // keep's op ran: the request branch is at try 2's tip and the attempts are gone; the request folder did not move.
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.tips[1].clone()), "{out}");
    assert!(fx.git.list_refs(TRY_PREFIX).unwrap().is_empty(), "{out}");
    assert!(!fx.try_folder(2).exists(), "{out}");
    assert_eq!(fs::read(fx.request.join("src/a/a.txt")).unwrap(), b"a\n", "{out}");

    let repeated = fx.keep(2);
    let out = text(&repeated);
    assert_eq!(repeated.status.code(), Some(0), "repeat: {out}");
    assert_eq!(member(&repeated, "outcome"), "already-done", "repeat: {out}");
    assert_harness_next(&member(&repeated, "next"));
    fx.assert_kept_two(&out);
    let all = ops(&fx.common);
    assert_ops_by_actor(&all[before..], &out);
}

#[test]
#[ignore = "worker execs mortar under the kill gate"]
fn keep_worker() {
    let root = PathBuf::from(std::env::var_os("MORTAR_TEST_ROOT").unwrap());
    let request = PathBuf::from(std::env::var_os("MORTAR_TEST_REQUEST").unwrap());
    let shimbin = PathBuf::from(std::env::var_os("MORTAR_TEST_SHIMBIN").unwrap());
    let path = std::env::join_paths(
        std::iter::once(shimbin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let git = fixture_git(&root);
    let mut mortar = command(&root, &request, &git);
    mortar.args(["--actor", ACTOR, "--json", "folder", "keep", "2"]).env("PATH", path)
        .env("MORTAR_TEST_WORKER_PID", std::process::id().to_string())
        .env("MORTAR_TEST_CRASH", std::env::var_os("MORTAR_TEST_CRASH").unwrap());
    for name in ["MORTAR_TEST_GIT_PATH", "MORTAR_TEST_GIT_SHIM", "TESTKIT_GATE_RELEASE", "TESTKIT_GATE_ACK", "TESTKIT_GATE_ARMED"] {
        mortar.env(name, std::env::var_os(name).unwrap());
    }
    panic!("exec mortar: {}", mortar.exec());
}
