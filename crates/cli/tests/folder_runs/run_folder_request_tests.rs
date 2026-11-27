//! `mortar folder request`: sparse paths and root files, build output sharing and its report, missing --paths,
//! and a zone with no branch. Drives the mortar binary against real git and throwaway repositories.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::support::{Config, ZoneConfig, write_fixture_config};
use folders::make::read_folder_record;
use folders::FolderKind;
use gitio::{Git, Oid, RunOpts};
use oplog::OpRecord;
use testkit::TestRepo;

const ACTOR: &str = "harness";
const USAGE: &str = "mr folder (zone <zone> <path> [--start <commit>] | request <zone> <request> <path> --paths \
                     <path>... [--with <zone>]... | remove <path> | attempts <n> <dir> | keep <k> | drop | refresh <zone \
                     folder path>) --actor <name> [--key <operation key>] [--json]";
const REQUEST: &str = "refs/heads/req/core/task";

struct Fixture {
    repo: TestRepo,
    git: Git,
    common: PathBuf,
    base: Oid,
    path: PathBuf,
}

impl Fixture {
    /// A commit holding Cargo.toml, src/a/a.txt and src/b/b.txt; zone core owns src, Cargo.toml is the root file.
    /// With `zone_branch`, zone/core exists at the commit. No build dirs are passed, so mr init writes its
    /// default ["target"].
    fn new(zone_branch: bool) -> Self {
        Self::with_build_dirs(zone_branch, &[])
    }

    fn with_build_dirs(zone_branch: bool, build_dirs: &[&str]) -> Self {
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
            zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src".into()] })]),
            root_files: vec!["Cargo.toml".into()], build_dirs: build_dirs.iter().map(|dir| (*dir).into()).collect(),
            cap_bytes: snapshot::DEFAULT_SIZE_CAP,
            floor_bytes: 0, formatter: None,
        }).unwrap();
        let path = repo.path.join("request-folder");
        Self { repo, git, common, base, path }
    }

    /// `mortar --actor harness --json folder <words>`, run in the repository's main checkout.
    fn folder(&self, words: &[&str]) -> Output {
        command(&self.repo.path, &self.repo.path, &self.git)
            .args(["--actor", ACTOR, "--json", "folder"]).args(words).output().unwrap()
    }

    fn zone_folder(&self) -> PathBuf {
        let path = self.repo.path.join("zone-folder");
        let made = self.folder(&["zone", "core", path.to_str().unwrap()]);
        assert_eq!(made.status.code(), Some(0), "{}", text(&made));
        path
    }

    fn request(&self) -> Output {
        self.folder(&["request", "core", "task", self.path_arg(), "--paths", "src/a"])
    }

    fn path_arg(&self) -> &str {
        self.path.to_str().unwrap()
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
        .env("CODETEAM_SESSION", "folder-request-test").stdin(Stdio::null());
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

fn assert_build_output(output: &Output, expected: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.ends_with(&format!(",\"build_output\":{expected}}}\n")), "{stdout}");
}

fn built_target(zone: &Path) {
    fs::create_dir_all(zone.join("target/nested")).unwrap();
    fs::write(zone.join("target/object"), [0, 1, 255, 42]).unwrap();
    fs::write(zone.join("target/nested/output"), b"built bytes\n").unwrap();
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
fn request_folder_holds_only_its_paths_and_the_root_files() {
    let fx = Fixture::new(true);
    let before = ops(&fx.common).len();
    let made = fx.folder(&["request", "core", "task", fx.path_arg(), "--paths", "src/a"]);
    let out = text(&made);
    assert_eq!(made.status.code(), Some(0), "{out}");
    assert_eq!(member(&made, "outcome"), "made", "{out}");
    let report = crate::json::check(made.clone(), "folder", 0, "made");
    assert_eq!(report.strings("lines"), [
        format!("the request folder of request task in zone core is at {}", fx.path.display()),
        format!("base {}", fx.base.as_str()),
        "cone Cargo.toml, src/a".to_string(),
        "build dir target not shared: found no zone folder of zone core".to_string(),
    ]);
    assert_build_output(&made, r#"[{"dir":"target","outcome":"refused","why":"found no zone folder of zone core"}]"#);
    let next = member(&made, "next");
    assert!(next.starts_with("start the request's agent in "), "{out}");
    assert!(next.contains("request-folder"), "{out}");
    assert_harness_next(&next);
    // The zone owns all of src; the request folder holds only its own path and the root file.
    assert_eq!(files(&fx.path), ["Cargo.toml", "src/a/a.txt"], "{out}");
    let record = read_folder_record(&fx.git.git_dir(&fx.path).unwrap()).unwrap().unwrap();
    assert_eq!(record.kind, FolderKind::Request, "{out}");
    assert_eq!(record.zone, "core");
    assert_eq!(record.request.as_deref(), Some("task"));
    assert_eq!(record.base, fx.base);
    assert_eq!(record.work_order_paths, ["Cargo.toml", "src/a"]);
    assert_eq!(record.parent.as_deref(), Some("zone/core"));
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), Some(fx.base.clone()), "{out}");
    let all = ops(&fx.common);
    let logged = &all[before..];
    assert!(
        logged.iter().any(|r| r.kind == "folder-branch" && r.moves.iter().any(|m| m.refname == REQUEST && m.new == fx.base)),
        "no folder-branch op made the request branch: {logged:?}\n{out}"
    );
    assert_ops_by_actor(logged, &out);
}

#[test]
fn request_folder_build_output_shared() {
    let fx = Fixture::with_build_dirs(true, &["target"]);
    let zone = fx.zone_folder();
    built_target(&zone);
    let made = fx.request();
    let report = crate::json::check(made.clone(), "folder", 0, "made");
    assert_eq!(report.strings("lines").len(), 3);
    assert_build_output(&made, r#"[{"dir":"target","outcome":"shared"}]"#);
    let source = zone.join("target");
    let dest = fx.path.join("target");
    assert_eq!(files(&source), ["nested/output", "object"]);
    assert_eq!(files(&dest), files(&source));
    for file in files(&source) {
        assert_eq!(fs::read(dest.join(&file)).unwrap(), fs::read(source.join(&file)).unwrap());
    }
}

#[test]
fn request_folder_build_output_absent() {
    let fx = Fixture::with_build_dirs(true, &["target", "build"]);
    let zone = fx.zone_folder();
    built_target(&zone);
    let made = fx.request();
    let report = crate::json::check(made.clone(), "folder", 0, "made");
    assert_eq!(&report.strings("lines")[3..], ["build dir build not shared: the zone folder has none"]);
    assert_build_output(&made, r#"[{"dir":"target","outcome":"shared"},{"dir":"build","outcome":"absent"}]"#);
    assert!(!fx.path.join("build").exists());
}

#[test]
fn request_folder_build_output_kept_on_repeat() {
    let fx = Fixture::with_build_dirs(true, &["target"]);
    let zone = fx.zone_folder();
    built_target(&zone);
    let first = fx.request();
    assert_eq!(first.status.code(), Some(0), "{}", text(&first));
    assert!(fx.path.join("target/object").exists(), "first request did not share target");
    fs::write(fx.path.join("target/object"), b"request changes").unwrap();
    let made = fx.request();
    let report = crate::json::check(made.clone(), "folder", 0, "made");
    assert_eq!(&report.strings("lines")[3..], ["build dir target not shared: the folder already has it, kept as it is"]);
    assert_build_output(&made, r#"[{"dir":"target","outcome":"kept"}]"#);
    assert_eq!(fs::read(fx.path.join("target/object")).unwrap(), b"request changes");
}

#[test]
fn request_folder_build_output_refused_without_failing() {
    let fx = Fixture::with_build_dirs(true, &["target"]);
    let zone = fx.zone_folder();
    let elsewhere = fx.repo.path.join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, zone.join("target")).unwrap();
    let made = fx.request();
    let report = crate::json::check(made.clone(), "folder", 0, "made");
    assert_eq!(&report.strings("lines")[3..], [
        "build dir target not shared: the parent's build dir \"target\" is a symlink; a clone would point into the parent",
    ]);
    assert_build_output(&made, r#"[{"dir":"target","outcome":"refused","why":"the parent's build dir \"target\" is a symlink; a clone would point into the parent"}]"#);
    assert!(!fx.path.join("target").exists());
}

#[test]
fn without_paths_it_is_a_usage_error_and_makes_nothing() {
    let fx = Fixture::new(true);
    let worktrees = fx.worktrees();
    let refused = fx.folder(&["request", "core", "task", fx.path_arg()]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(2), "{out}");
    assert_eq!(member(&refused, "outcome"), "usage", "{out}");
    assert_eq!(member(&refused, "next"), USAGE, "{out}");
    assert!(!fx.path.exists(), "{out}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), None, "{out}");
    assert_eq!(fx.worktrees(), worktrees, "{out}");
}

#[test]
fn a_zone_with_no_branch_is_refused_and_makes_nothing() {
    let fx = Fixture::new(false);
    let before = ops(&fx.common).len();
    let worktrees = fx.worktrees();
    let refused = fx.folder(&["request", "core", "task", fx.path_arg(), "--paths", "src/a"]);
    let out = text(&refused);
    assert_eq!(refused.status.code(), Some(3), "{out}");
    assert_eq!(member(&refused, "outcome"), "refused", "{out}");
    let next = member(&refused, "next");
    assert!(next.contains("mr folder zone"), "{out}");
    assert_harness_next(&next);
    assert!(!fx.path.exists(), "{out}");
    assert_eq!(fx.git.read_ref(REQUEST).unwrap(), None, "{out}");
    assert_eq!(fx.worktrees(), worktrees, "{out}");
    assert_eq!(ops(&fx.common).len(), before, "{out}");
}
