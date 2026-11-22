//! `mortar folder zone`: a sparse zone folder from the config, its repeat, an unknown zone, a missing --actor, and
//! --start making a new zone branch (or naming no commit). The no-start cases prove the origin tip, not HEAD,
//! supplies a missing zone branch, and a missing origin is refused with --start next.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.
//! Tests to write (not written here):
//! - a folders lock held by a live pid: exit 3 naming the holder's pid, host and since (cli with oplog's locks);
//! - free disk under the floor: exit 1 with the cli's own facts, never folders' BelowFloor Display text;
//! - the zone branch checked out in another folder, or a different folder at the path: the cli's why and next,
//!   never folders' FolderError Display text.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::support::{Config, ZoneConfig, write_fixture_config};
use folders::make::read_folder_record;
use folders::{FolderKind, FolderRecord};
use gitio::{Git, Oid, RunOpts};
use oplog::OpRecord;
use testkit::TestRepo;

const ACTOR: &str = "harness";
const USAGE: &str = "mr folder (zone <zone> <path> [--start <commit>] | request <zone> <request> <path> --paths \
                     <path>... | remove <path> | attempts <n> <dir> | keep <k> | drop | refresh <path>) --actor <name> [--key \
                     <operation key>] [--json]";

struct Fixture {
    repo: TestRepo,
    git: Git,
    common: PathBuf,
    base: Oid,
    path: PathBuf,
}

impl Fixture {
    /// A commit holding Cargo.toml, src/a/a.txt and src/b/b.txt; zone core owns src/a, Cargo.toml is the root file.
    /// With `zone_branch`, zone/core exists at the commit.
    fn new(zone_branch: bool) -> Self {
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
        if zone_branch {
            git.run(&["branch", "zone/core", base.as_str()], &RunOpts::default()).unwrap();
        }
        let common = git.common_dir().unwrap();
        write_fixture_config(&git.repo, &Config {
            origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
            zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src/a".into()] })]),
            root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
            floor_bytes: 0, formatter: None,
        }).unwrap();
        let path = repo.path.join("zone-folder");
        Self { repo, git, common, base, path }
    }

    /// `mortar --json [--actor harness] folder <words>`, run in the repository's main checkout.
    fn folder(&self, actor: bool, words: &[&str]) -> Output {
        let mut command = command(&self.repo.path, &self.repo.path, &self.git);
        if actor {
            command.args(["--actor", ACTOR]);
        }
        command.args(["--json", "folder"]).args(words).output().unwrap()
    }

    fn path_arg(&self) -> &str {
        self.path.to_str().unwrap()
    }

    fn record(&self) -> FolderRecord {
        read_folder_record(&self.git.git_dir(&self.path).unwrap()).unwrap().unwrap()
    }

    fn worktrees(&self) -> usize {
        self.git.worktree_list().unwrap().len()
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
        .env("CODETEAM_SESSION", "folder-zone-test").stdin(Stdio::null());
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

fn assert_ops_by_actor(records: &[OpRecord], out: &str) {
    for record in records {
        assert_eq!(record.actor, ACTOR, "op {} ({}) logged by another actor: {out}", record.op_id, record.kind);
    }
}

#[test]
fn zone_folder_holds_only_the_zone_paths_and_root_files_and_a_repeat_finishes() {
    let fx = Fixture::new(true);
    let before = ops(&fx.common).len();
    let made = fx.folder(true, &["zone", "core", fx.path_arg()]);
    let out = text(&made);
    assert_eq!(made.status.code(), Some(0), "{out}");
    assert_eq!(member(&made, "outcome"), "made", "{out}");
    let next = member(&made, "next");
    assert!(next.starts_with("make request folders with mr folder request "), "{out}");
    assert!(next.contains("--paths"), "{out}");
    assert_harness_next(&next);
    assert_eq!(files(&fx.path), ["Cargo.toml", "src/a/a.txt"], "{out}");
    let record = fx.record();
    assert_eq!(record.kind, FolderKind::Zone, "{out}");
    assert_eq!(record.zone, "core");
    assert_eq!(record.request, None);
    assert_eq!(record.base, fx.base);
    assert_eq!(record.work_order_paths, ["Cargo.toml", "src/a"]);
    assert_eq!(fx.git.read_ref("refs/heads/zone/core").unwrap(), Some(fx.base.clone()));
    assert_ops_by_actor(&ops(&fx.common)[before..], &out);

    let repeated = fx.folder(true, &["zone", "core", fx.path_arg()]);
    let out = text(&repeated);
    assert_eq!(repeated.status.code(), Some(0), "repeat: {out}");
    assert_eq!(fx.record(), record, "repeat: {out}");
    assert_eq!(files(&fx.path), ["Cargo.toml", "src/a/a.txt"], "repeat: {out}");
    assert_ops_by_actor(&ops(&fx.common)[before..], &out);
}

#[test]
fn a_zone_not_in_the_config_is_refused_and_makes_nothing() {
    let fx = Fixture::new(true);
    let before = ops(&fx.common).len();
    let worktrees = fx.worktrees();
    let refused = fx.folder(true, &["zone", "web", fx.path_arg()]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert!(next.contains("mr init --replace"), "{out}");
    assert!(next.contains("--zone web="), "{out}");
    assert_harness_next(&next);
    assert!(!fx.path.exists(), "{out}");
    assert_eq!(fx.git.read_ref("refs/heads/zone/web").unwrap(), None, "{out}");
    assert_eq!(fx.worktrees(), worktrees, "{out}");
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}

#[test]
fn without_an_actor_it_is_a_usage_error_and_makes_nothing() {
    let fx = Fixture::new(true);
    let worktrees = fx.worktrees();
    let refused = fx.folder(false, &["zone", "core", fx.path_arg()]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(4), "{out}");
    assert_eq!(member(&refused, "outcome"), "usage", "{out}");
    assert_eq!(member(&refused, "next"), USAGE, "{out}");
    assert!(!fx.path.exists(), "{out}");
    assert_eq!(fx.worktrees(), worktrees, "{out}");
}

#[test]
fn start_on_a_new_zone_makes_its_branch_at_the_commit_by_an_op_of_the_actor() {
    let fx = Fixture::new(false);
    let before = ops(&fx.common).len();
    let made = fx.folder(true, &["zone", "core", fx.path_arg(), "--start", "main"]);
    let out = text(&made);
    assert_eq!(made.status.code(), Some(0), "{out}");
    assert_eq!(member(&made, "outcome"), "made", "{out}");
    assert_harness_next(&member(&made, "next"));
    assert_eq!(fx.git.read_ref("refs/heads/zone/core").unwrap(), Some(fx.base.clone()), "{out}");
    assert_eq!(files(&fx.path), ["Cargo.toml", "src/a/a.txt"], "{out}");
    assert_eq!(fx.record().base, fx.base, "{out}");
    let all = ops(&fx.common);
    let logged = &all[before..];
    assert!(
        logged.iter().any(|r| r.kind == "folder-branch"
            && r.moves.iter().any(|m| m.refname == "refs/heads/zone/core" && m.new == fx.base)),
        "no folder-branch op made zone/core: {logged:?}\n{out}"
    );
    assert_ops_by_actor(logged, &out);
}

#[test]
fn precheck_scope_a_bare_zone_starts_at_the_origin_tip_not_the_callers_detached_head() {
    let fx = Fixture::new(false);
    fx.git.run(&["checkout", "--detach", "-q", fx.base.as_str()], &RunOpts::default()).unwrap();
    let tree = fx.git.commit_info(&fx.base).unwrap().tree;
    let origin = fx.git.commit_tree(&tree, std::slice::from_ref(&fx.base), "new origin tip\n", None, None).unwrap();
    fx.git.run(&["update-ref", "refs/heads/main", origin.as_str()], &RunOpts::default()).unwrap();
    assert_ne!(origin, fx.base); assert_eq!(fx.git.rev_parse("HEAD").unwrap(), Some(fx.base.clone()));
    let before = ops(&fx.common).len();
    let made = fx.folder(true, &["zone", "core", fx.path_arg()]); let out = text(&made);
    assert_eq!(made.status.code(), Some(0), "{out}"); assert_eq!(member(&made, "outcome"), "made");
    assert_harness_next(&member(&made, "next"));
    assert_eq!(fx.git.read_ref("refs/heads/zone/core").unwrap(), Some(origin.clone()));
    assert_eq!(fx.record().base, origin); assert_eq!(files(&fx.path), ["Cargo.toml", "src/a/a.txt"]);
    let all = ops(&fx.common); let added = &all[before..]; assert_ops_by_actor(added, &out);
    assert!(added.iter().any(|r| r.kind == "folder-branch" && r.phase == oplog::OpPhase::Done
        && r.moves.iter().any(|m| m.refname == "refs/heads/zone/core" && m.old.is_none() && m.new == origin)));
}

#[test]
fn precheck_scope_a_bare_zone_without_an_origin_or_zone_branch_refuses_naming_start_and_makes_nothing() {
    let fx = Fixture::new(false);
    fx.git.run(&["update-ref", "-d", "refs/heads/main"], &RunOpts::default()).unwrap();
    assert_eq!(fx.git.read_ref("refs/heads/main").unwrap(), None);
    let before = ops(&fx.common); let worktrees = fx.worktrees();
    let refused = fx.folder(true, &["zone", "core", fx.path_arg()]); let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}"); assert_eq!(member(&refused, "outcome"), "refused");
    let next = member(&refused, "next"); assert!(next.contains("--start"), "{out}"); assert_harness_next(&next);
    assert!(!fx.path.exists()); assert_eq!(fx.git.read_ref("refs/heads/zone/core").unwrap(), None);
    assert_eq!(fx.git.read_ref("refs/heads/main").unwrap(), None);
    assert_eq!(fx.worktrees(), worktrees); assert_eq!(ops(&fx.common), before);
}

#[test]
fn a_start_that_names_no_commit_is_refused_and_makes_nothing() {
    let fx = Fixture::new(false);
    let before = ops(&fx.common).len();
    let worktrees = fx.worktrees();
    let refused = fx.folder(true, &["zone", "core", fx.path_arg(), "--start", "no-such-commit"]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(1), "{out}");
    assert_harness_next(&member(&refused, "next"));
    assert!(!fx.path.exists(), "{out}");
    assert_eq!(fx.git.read_ref("refs/heads/zone/core").unwrap(), None, "{out}");
    assert_eq!(fx.worktrees(), worktrees, "{out}");
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}
