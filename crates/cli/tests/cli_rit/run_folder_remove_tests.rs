//! `mortar folder remove`: a joined, saved request folder removed with its branch and snapshot ref; unsaved work,
//! remaining attempts and a zone folder each refused with nothing removed.
//! RIT (S-135, S-136): drives the mortar binary against real git and the crates below cli; owned by Nagatha from
//! landing.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-104, S-109):
//! - a file the snapshot cap refuses counts as unsaved (S-32): exit 1, the cli's facts naming the file and the cap
//!   (cli with snapshot's saved_state);
//! - a request not joined (NotJoined) and a snapshot ref moved under the op (RemovedRefsLeft): the cli's why and
//!   next, never folders' FolderError Display text;
//! - a join-<zone> lock held by a live pid: exit 3 naming the holder's pid, host and since (cli with oplog's locks).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::{Config, ZoneConfig};
use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::{Git, Oid, RunOpts};
use oplog::{KillPidProbe, OpRecord};
use testkit::TestRepo;

const ACTOR: &str = "harness";
const REQUEST: &str = "refs/heads/req/core/task";
const SNAP: &str = "refs/mortar/snap/core/task";

struct Fixture {
    repo: TestRepo,
    git: Git,
    common: PathBuf,
    request: PathBuf,
    request_tip: Oid,
    snap: Oid,
}

impl Fixture {
    /// zone/core at a commit holding Cargo.toml, src/a/a.txt and src/b/b.txt; request folder core/task on src/a and
    /// Cargo.toml, saved by `mortar save` with nothing changed, so its branch is still the zone's tip (joined).
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
            zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src/a".into()] })]),
            root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
            floor_bytes: 0, formatter: None,
        }).unwrap();
        let request = repo.path.join("request-folder");
        oplog::recover(&git, &common).unwrap();
        folders::make_request_folder(&setup(&git, &request), "core", "task", &paths()).unwrap();
        let saved = command(&repo.path, &request, &git).args(["save", "Start the task"]).output().unwrap();
        assert!(saved.status.success(), "fixture save: {}", text(&saved));
        let request_tip = git.read_ref(REQUEST).unwrap().unwrap();
        assert_eq!(request_tip, base, "a save with nothing changed keeps the request at the zone's tip");
        let snap = git.read_ref(SNAP).unwrap().unwrap();
        Self { repo, git, common, request, request_tip, snap }
    }

    /// `mortar --actor harness --json folder remove <path>`, run in the repository's main checkout.
    fn remove(&self, path: &Path) -> Output {
        command(&self.repo.path, &self.repo.path, &self.git)
            .args(["--actor", ACTOR, "--json", "folder", "remove", path.to_str().unwrap()]).output().unwrap()
    }

    /// The request folder, its branch and its snapshot ref are as the fixture left them.
    fn assert_request_untouched(&self, out: &str) {
        assert!(self.request.join("src/a/a.txt").exists(), "{out}");
        assert_eq!(self.git.read_ref(REQUEST).unwrap(), Some(self.request_tip.clone()), "{out}");
        assert_eq!(self.git.read_ref(SNAP).unwrap(), Some(self.snap.clone()), "{out}");
    }
}

fn paths() -> Vec<String> {
    vec!["src/a".into(), "Cargo.toml".into()]
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
        .env("CODETEAM_SESSION", "folder-remove-test").stdin(Stdio::null());
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
    for agent_verb in ["mortar save", "mortar join", "mortar need", "mortar resolve", "mortar status",
                       "mr save", "mr join", "mr need", "mr resolve", "mr status"] {
        assert!(!next.contains(agent_verb), "next names an agent verb: {next}");
    }
    assert!(!next.contains("agent run"), "next is aimed at an agent: {next}");
}

fn ops(common: &Path) -> Vec<OpRecord> {
    oplog::read_op_log(common).unwrap().records
}

#[test]
fn a_joined_saved_request_folder_is_removed_with_its_branch_and_snapshot_ref() {
    let fx = Fixture::new();
    let before = ops(&fx.common).len();
    let removed = fx.remove(&fx.request);
    let out = text(&removed);
    assert_eq!(removed.status.code(), Some(0), "{out}");
    assert_eq!(member(&removed, "outcome"), "removed", "{out}");
    let next = member(&removed, "next");
    assert_eq!(next, "nothing more for this request", "{out}");
    assert_harness_next(&next);
    assert!(!fx.request.exists(), "{out}");
    assert_eq!(fx.git.worktree_list().unwrap().len(), 1, "only the main checkout is left: {out}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), None, "{out}");
    assert_eq!(fx.git.read_ref(SNAP).unwrap(), None, "{out}");
    let all = ops(&fx.common);
    let logged = &all[before..];
    assert!(logged.iter().any(|r| r.kind == "folder-remove"), "no folder-remove op: {logged:?}\n{out}");
    for record in logged {
        assert_eq!(record.actor, ACTOR, "op {} ({}) logged by another actor: {out}", record.op_id, record.kind);
    }
}

#[test]
fn a_request_folder_with_unsaved_work_is_refused_and_nothing_is_removed() {
    let fx = Fixture::new();
    fs::write(fx.request.join("src/a/a.txt"), b"unsaved\n").unwrap();
    let before = ops(&fx.common).len();
    let refused = fx.remove(&fx.request);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert_eq!(next, "repeat once the folder's work is saved", "{out}");
    assert_harness_next(&next);
    fx.assert_request_untouched(&out);
    assert_eq!(fs::read(fx.request.join("src/a/a.txt")).unwrap(), b"unsaved\n", "{out}");
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}

#[test]
fn a_request_folder_with_attempts_left_is_refused_naming_keep_and_drop() {
    let fx = Fixture::new();
    let tries = fx.repo.path.join("tries");
    oplog::recover(&fx.git, &fx.common).unwrap();
    folders::make_attempts(&setup(&fx.git, &tries), "core", "task", &paths(), 1).unwrap();
    let try_branch = fx.git.read_ref("refs/heads/try/core/task/1").unwrap();
    assert!(try_branch.is_some());
    let before = ops(&fx.common).len();
    let refused = fx.remove(&fx.request);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert!(next.contains("mr folder keep"), "{out}");
    assert!(next.contains("mr folder drop"), "{out}");
    assert_harness_next(&next);
    fx.assert_request_untouched(&out);
    assert!(tries.join("try-1").exists(), "{out}");
    assert_eq!(fx.git.read_ref("refs/heads/try/core/task/1").unwrap(), try_branch, "{out}");
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}

#[test]
fn a_zone_folder_is_refused_and_nothing_is_removed() {
    let fx = Fixture::new();
    let zone = fx.repo.path.join("zone-folder");
    oplog::recover(&fx.git, &fx.common).unwrap();
    folders::make_zone_folder(&setup(&fx.git, &zone), "core", &paths(), None).unwrap();
    let zone_tip = fx.git.read_ref("refs/heads/zone/core").unwrap();
    let before = ops(&fx.common).len();
    let refused = fx.remove(&zone);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    assert_harness_next(&member(&refused, "next"));
    assert!(zone.join("src/a/a.txt").exists(), "{out}");
    assert_eq!(fx.git.read_ref("refs/heads/zone/core").unwrap(), zone_tip, "{out}");
    fx.assert_request_untouched(&out);
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}
