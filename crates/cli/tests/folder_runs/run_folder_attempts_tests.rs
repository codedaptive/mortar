//! `mortar folder attempts <n> <dir>`: attempt folders made from a request folder at <dir>/try-<k>, each on its try
//! branch at the request's tip; run in a zone folder it is refused, and n = 0 is a usage error. A relative <dir> is
//! taken from the request folder, not the subfolder it is run in, and one that normalizes inside it is a usage error.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.
//! Tests to write (not written here):
//! - a folders lock held by a live pid: exit 3 naming the holder's pid, host and since (cli with oplog's locks);
//! - free disk under the floor at attempt j: exit 1 with the cli's own facts, never folders' BelowFloor Display text,
//!   and a repeat making the rest;
//! - a request branch gone (RequestBranchMissing): the cli's why and next, never folders' FolderError Display text.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::support::{Config, ZoneConfig, write_fixture_config};
use folders::make::read_folder_record;
use folders::{DfFreeDisk, FolderKind, FolderSetup, SweepEstimate};
use gitio::{Git, Oid, RunOpts};
use oplog::{KillPidProbe, OpPhase, OpRecord};
use testkit::TestRepo;

const ACTOR: &str = "harness";
const USAGE: &str = "mr folder (zone <zone> <path> [--start <commit>] | request <zone> <request> <path> --paths \
                     <path>... [--with <zone>]... | remove <path> | attempts <n> <dir> | keep <k> | drop | refresh <zone \
                     folder path>) --actor <name> [--key <operation key>] [--json]";
const TRY_PREFIX: &str = "refs/heads/try/core/task/";

struct Fixture {
    repo: TestRepo,
    git: Git,
    common: PathBuf,
    request: PathBuf,
    request_tip: Oid,
    dir: PathBuf,
}

impl Fixture {
    /// zone/core at a commit holding Cargo.toml, src/a/a.txt and src/b/b.txt; request folder core/task on src/a and
    /// Cargo.toml. The attempts' directory `dir` is outside the request folder and not made yet.
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
        write_fixture_config(&git.repo, &Config {
            origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
            zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src".into()] })]),
            root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
            floor_bytes: 0, formatter: None,
        }).unwrap();
        let request = repo.path.join("request-folder");
        oplog::recover(&git, &common).unwrap();
        folders::make_request_folder(&setup(&git, &request), "core", "task", &["src/a".into(), "Cargo.toml".into()])
            .unwrap();
        let request_tip = git.read_ref("refs/heads/req/core/task").unwrap().unwrap();
        let dir = repo.path.join("tries");
        Self { repo, git, common, request, request_tip, dir }
    }

    /// `mortar --actor harness --json folder <words>`, run in `cwd`.
    fn folder(&self, cwd: &Path, words: &[&str]) -> Output {
        command(&self.repo.path, cwd, &self.git).args(["--actor", ACTOR, "--json", "folder"]).args(words).output().unwrap()
    }

    fn dir_arg(&self) -> &str {
        self.dir.to_str().unwrap()
    }

    /// Nothing of an attempt exists: no directory, no try branch, no op since `before`.
    fn assert_nothing_made(&self, before: usize, out: &str) {
        assert!(!self.dir.exists(), "{out}");
        assert!(self.git.list_refs(TRY_PREFIX).unwrap().is_empty(), "{out}");
        assert_eq!(ops(&self.common).len(), before, "{out}");
    }
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
        .env("CODETEAM_SESSION", "folder-attempts-test").stdin(Stdio::null());
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

/// A harness verb's next step is never aimed at an agent: it names no agent verb and asks no agent to
/// run anything.
fn assert_harness_next(next: &str) {
    for agent_verb in ["mortar save", "mortar join", "mortar need", "mortar resolve", "mortar status",
                       "mr save", "mr join", "mr need", "mr resolve", "mr status"] {
        assert!(!next.contains(agent_verb), "next names an agent verb: {next}");
    }
    assert!(!next.contains("agent run"), "next is aimed at an agent: {next}");
}

/// Every file under `dir` (its `.git` file left out), relative, sorted.
fn files(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().is_some_and(|name| name == ".git") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                found.push(path.strip_prefix(dir).unwrap().display().to_string());
            }
        }
    }
    found.sort();
    found
}

fn ops(common: &Path) -> Vec<OpRecord> {
    oplog::read_op_log(common).unwrap().records
}

#[test]
fn attempts_are_made_at_dir_try_k_on_try_branches_at_the_request_tip() {
    let fx = Fixture::new();
    let before = ops(&fx.common).len();
    let made = fx.folder(&fx.request, &["attempts", "2", fx.dir_arg()]);
    let out = text(&made);
    assert_eq!(made.status.code(), Some(0), "{out}");
    assert_eq!(member(&made, "outcome"), "made", "{out}");
    let next = member(&made, "next");
    assert_eq!(
        next,
        "start one agent in each attempt folder; then mr folder keep <k> --actor <name> or mr folder drop \
         --actor <name> in the request's folder",
        "{out}"
    );
    assert_harness_next(&next);
    for k in 1..=2u32 {
        let folder = fx.dir.join(format!("try-{k}"));
        assert!(folder.exists(), "try-{k}: {out}");
        assert_eq!(files(&folder), ["Cargo.toml", "src/a/a.txt"], "try-{k}: {out}");
        let record = read_folder_record(&fx.git.git_dir(&folder).unwrap()).unwrap().unwrap();
        assert_eq!(record.kind, FolderKind::Attempt(k), "{out}");
        assert_eq!(record.zone, "core");
        assert_eq!(record.request.as_deref(), Some("task"));
        assert_eq!(record.base, fx.request_tip);
        let branch = format!("{TRY_PREFIX}{k}");
        assert_eq!(fx.git.read_ref(&branch).unwrap(), Some(fx.request_tip.clone()), "{branch}: {out}");
    }
    assert!(!fx.dir.join("try-3").exists(), "{out}");
    let all = ops(&fx.common);
    let logged = &all[before..];
    // The log holds an Intent and a Done record per op: two ops are two Done records.
    let done = logged.iter().filter(|r| r.kind == "folder-branch" && r.phase == OpPhase::Done).count();
    assert_eq!(done, 2, "{logged:?}\n{out}");
    for record in logged {
        assert_eq!(record.actor, ACTOR, "op {} ({}) logged by another actor: {out}", record.op_id, record.kind);
    }
}

#[test]
fn attempts_run_in_a_zone_folder_are_refused_and_make_nothing() {
    let fx = Fixture::new();
    let zone = fx.repo.path.join("zone-folder");
    oplog::recover(&fx.git, &fx.common).unwrap();
    folders::make_zone_folder(&setup(&fx.git, &zone), "core", &["src".into(), "Cargo.toml".into()], None).unwrap();
    let before = ops(&fx.common).len();
    let refused = fx.folder(&zone, &["attempts", "2", fx.dir_arg()]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert_eq!(next, "run mr folder attempts <n> <dir> --actor <name> in the request's own folder", "{out}");
    assert_harness_next(&next);
    fx.assert_nothing_made(before, &out);
}

#[test]
fn zero_attempts_is_a_usage_error_and_makes_nothing() {
    let fx = Fixture::new();
    let before = ops(&fx.common).len();
    let refused = fx.folder(&fx.request, &["attempts", "0", fx.dir_arg()]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(4), "{out}");
    assert_eq!(member(&refused, "outcome"), "usage", "{out}");
    assert_eq!(member(&refused, "next"), USAGE, "{out}");
    fx.assert_nothing_made(before, &out);
}

#[test]
fn a_relative_dir_run_from_a_subfolder_is_taken_from_the_request_folder() {
    let fx = Fixture::new();
    let subfolder = fx.request.join("src/a");
    let made = fx.folder(&subfolder, &["attempts", "2", "../outside"]);
    let out = text(&made);
    assert_eq!(made.status.code(), Some(0), "{out}");
    assert_eq!(member(&made, "outcome"), "made", "{out}");
    // `../outside` from the request folder is the request folder's parent, the repository's top here.
    for k in 1..=2u32 {
        let folder = fx.repo.path.join(format!("outside/try-{k}"));
        assert_eq!(files(&folder), ["Cargo.toml", "src/a/a.txt"], "try-{k}: {out}");
    }
    assert!(!fx.request.join("src/outside").exists(), "{out}");
    assert!(!subfolder.join("outside").exists(), "{out}");
}

#[test]
fn a_relative_dir_that_normalizes_inside_the_request_folder_is_a_usage_error() {
    let fx = Fixture::new();
    let subfolder = fx.request.join("src/a");
    let before = ops(&fx.common).len();
    let refused = fx.folder(&subfolder, &["attempts", "2", "sub/../tries"]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(4), "{out}");
    assert_eq!(member(&refused, "outcome"), "usage", "{out}");
    assert!(!fx.request.join("tries").exists(), "{out}");
    assert!(!subfolder.join("tries").exists(), "{out}");
    fx.assert_nothing_made(before, &out);
}
