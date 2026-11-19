//! `mortar folder drop`: saved attempts dropped with the request branch unmoved and their snapshots kept; a
//! branch-only attempt whose trysnap tree equals its tip's tree dropped; one with no trysnap ref refused
//! naming its branch, with nothing changed.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.
//! Tests to write (not written here):
//! - a try branch moved under the op (AttemptRefsLeft): the cli's why and next, never folders' FolderError Display
//!   text;
//! - an orphan attempt folder with no trysnap ref (OrphanWithoutSnapshot): exit 1 naming the folder, nothing changed;
//! - a join-<zone> lock held by a live pid: exit 3 naming the holder's pid, host and since (cli with oplog's locks).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::support::{Config, ZoneConfig, write_fixture_config};
use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::{Git, Oid, RunOpts};
use oplog::{KillPidProbe, OpRecord};
use testkit::TestRepo;

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
}

impl Fixture {
    /// zone/core at a commit holding Cargo.toml, src/a/a.txt ("a") and src/b/b.txt; request folder core/task on src/a
    /// and Cargo.toml, saved with nothing changed. No attempts yet.
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
        folders::make_request_folder(&setup(&git, &request), "core", "task", &paths()).unwrap();
        save(&repo.path, &request, &git, "Start the task");
        let request_tip = git.read_ref(REQUEST).unwrap().unwrap();
        let dir = repo.path.join("tries");
        Self { repo, git, common, request, dir, request_tip }
    }

    /// Attempts 1 and 2 at <repo>/tries, attempt k with src/a/a.txt changed and saved by `mortar save` in its folder;
    /// answers their trysnap refs' values.
    fn saved_attempts(&self) -> Vec<Oid> {
        oplog::recover(&self.git, &self.common).unwrap();
        folders::make_attempts(&setup(&self.git, &self.dir), "core", "task", &paths(), 2).unwrap();
        (1..=2)
            .map(|k| {
                let folder = self.dir.join(format!("try-{k}"));
                fs::write(folder.join("src/a/a.txt"), format!("attempt {k}\n")).unwrap();
                save(&self.repo.path, &folder, &self.git, &format!("Attempt {k}"));
                self.git.read_ref(&format!("{TRYSNAP_PREFIX}{k}")).unwrap().unwrap()
            })
            .collect()
    }

    /// Attempt 1 as a branch only, no folder: try/core/task/1 at the request's tip.
    fn branch_only_attempt(&self) {
        let branch = format!("{TRY_PREFIX}1");
        self.git.run(&["update-ref", branch.as_str(), self.request_tip.as_str()], &RunOpts::default()).unwrap();
    }

    /// `mortar --actor harness --json folder drop`, run in the request folder.
    fn drop_attempts(&self) -> Output {
        command(&self.repo.path, &self.request, &self.git)
            .args(["--actor", ACTOR, "--json", "folder", "drop"]).output().unwrap()
    }

    /// The request branch and folder are as the fixture left them.
    fn assert_request_unmoved(&self, out: &str) {
        assert_eq!(self.git.read_ref(REQUEST).unwrap(), Some(self.request_tip.clone()), "{out}");
        assert_eq!(fs::read(self.request.join("src/a/a.txt")).unwrap(), b"a\n", "{out}");
    }
}

fn paths() -> Vec<String> {
    vec!["src/a".into(), "Cargo.toml".into()]
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
        .env("CODETEAM_SESSION", "folder-drop-test").stdin(Stdio::null());
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

fn ops(common: &Path) -> Vec<OpRecord> {
    oplog::read_op_log(common).unwrap().records
}

/// The dropped attempts' branches are gone by one folder-drop-attempts op, and every op since `before` names the
/// --actor.
fn assert_dropped_by_the_actor(common: &Path, before: usize, out: &str) {
    let all = ops(common);
    let logged = &all[before..];
    assert!(logged.iter().any(|r| r.kind == "folder-drop-attempts"), "no folder-drop-attempts op: {logged:?}\n{out}");
    for record in logged {
        assert_eq!(record.actor, ACTOR, "op {} ({}) logged by another actor: {out}", record.op_id, record.kind);
    }
}

#[test]
fn saved_attempts_are_dropped_with_the_request_unmoved_and_their_snapshots_kept() {
    let fx = Fixture::new();
    let trysnaps = fx.saved_attempts();
    let before = ops(&fx.common).len();
    let dropped = fx.drop_attempts();
    let out = text(&dropped);
    assert_eq!(dropped.status.code(), Some(0), "{out}");
    assert_eq!(member(&dropped, "outcome"), "dropped", "{out}");
    assert_harness_next(&member(&dropped, "next"));
    fx.assert_request_unmoved(&out);
    assert!(!fx.dir.join("try-1").exists(), "{out}");
    assert!(!fx.dir.join("try-2").exists(), "{out}");
    assert!(fx.git.list_refs(TRY_PREFIX).unwrap().is_empty(), "{out}");
    for (k, trysnap) in (1..=2).zip(&trysnaps) {
        assert_eq!(fx.git.read_ref(&format!("{TRYSNAP_PREFIX}{k}")).unwrap().as_ref(), Some(trysnap), "trysnap {k}: {out}");
    }
    assert_dropped_by_the_actor(&fx.common, before, &out);
}

#[test]
fn a_branch_only_attempt_whose_snapshot_tree_equals_its_tip_tree_is_dropped() {
    let fx = Fixture::new();
    fx.branch_only_attempt();
    // A snapshot commit other than the tip, holding the tip's tree: saved by the trees-equal rule, not by identity.
    let tree = fx.git.commit_info(&fx.request_tip).unwrap().tree;
    let made = fx.git.run(&["commit-tree", tree.as_str(), "-p", fx.request_tip.as_str(), "-m", "snapshot"], &RunOpts::default())
        .unwrap();
    let trysnap = String::from_utf8(made.stdout).unwrap().trim().to_string();
    assert_ne!(trysnap, fx.request_tip.as_str());
    let trysnap_ref = format!("{TRYSNAP_PREFIX}1");
    fx.git.run(&["update-ref", trysnap_ref.as_str(), trysnap.as_str()], &RunOpts::default()).unwrap();
    let before = ops(&fx.common).len();
    let dropped = fx.drop_attempts();
    let out = text(&dropped);
    assert_eq!(dropped.status.code(), Some(0), "{out}");
    assert_eq!(member(&dropped, "outcome"), "dropped", "{out}");
    assert_harness_next(&member(&dropped, "next"));
    fx.assert_request_unmoved(&out);
    assert_eq!(fx.git.read_ref(&format!("{TRY_PREFIX}1")).unwrap(), None, "{out}");
    let kept = fx.git.read_ref(&trysnap_ref).unwrap().map(|oid| oid.as_str().to_string());
    assert_eq!(kept, Some(trysnap), "{out}");
    assert_dropped_by_the_actor(&fx.common, before, &out);
}

#[test]
fn a_branch_only_attempt_with_no_snapshot_is_refused_naming_its_branch() {
    let fx = Fixture::new();
    fx.branch_only_attempt();
    let before = ops(&fx.common).len();
    let refused = fx.drop_attempts();
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    assert!(out.contains("try/core/task/1"), "the refusal names the branch: {out}");
    let next = member(&refused, "next");
    assert_eq!(next, "repeat once each listed attempt's work is saved", "{out}");
    assert_harness_next(&next);
    fx.assert_request_unmoved(&out);
    assert_eq!(fx.git.read_ref(&format!("{TRY_PREFIX}1")).unwrap(), Some(fx.request_tip.clone()), "{out}");
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}
