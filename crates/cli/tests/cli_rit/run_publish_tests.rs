//! `mortar publish`: a landing is put on the remote's main and then local main, and the squash ref removed; with no
//! landing there is nothing to publish and the remote is untouched; a remote main someone else moved exits 1
//! origin-moved, with local main and integration on the commit found, the integration worktree's files at it and the
//! squash ref kept; an extra word is a usage error (exit 4) with publish's usage line as next.
//! RIT (S-135, S-136): drives the mortar binary against real git and the crates below cli; owned by Nagatha from
//! landing. A bare repository in a second throwaway directory is the remote; the squash and the landing are made in
//! setup through land::squash and land::land under the land lock.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-135):
//! - the land lock held by a live pid: exit 3 (cli with oplog's locks);
//! - without a config: exit 1, next mortar init (the shell's load_config);
//! - each land error variant's text that needs git or the op log to fail: PublishError PushRefused, RemoteMissing
//!   and PushIntent, and LandError's;
//! - ACCEPT-1 checks 7, 9, 11 and 12 through these verbs on two zones;
//! - publish with --key: the key names the lock holder only (S-132).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::{Config, ZoneConfig};
use gitio::Git;
use crate::json::check;
use land::{LandAnswer, LandRefs, Squash, SquashAnswer};
use oplog::{KillPidProbe, LockName};
use testkit::TestRepo;

const USAGE: &str = "mortar publish --actor <name> [--key <operation key>] [--json]";
const SQUASH_A: &str = "refs/mortar/squash/a";
const X: &str = "x1\nx2\nx3\n";
const X_ZONE: &str = "zone edit of x1\nx2\nx3\n";
const OTHER: &str = "pushed by another clone\n";

const GIT_ENV: [(&str, &str); 6] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_AUTHOR_NAME", "Publish Verb"),
    ("GIT_AUTHOR_EMAIL", "publish-verb@example.invalid"),
    ("GIT_COMMITTER_NAME", "Publish Verb"),
    ("GIT_COMMITTER_EMAIL", "publish-verb@example.invalid"),
];

/// A throwaway repository and a second throwaway directory (both removed on drop).
struct Setup {
    _repo: TestRepo,
    _tmp: TestRepo,
    root: PathBuf,
    tmp: PathBuf,
    bare: PathBuf,
}

/// The repository (zones a and b; main's first commit holding src/a/x, src/b/y and Cargo.toml; integration at it;
/// zone/a one commit ahead editing src/a/x), with origin a bare repository at `<tmp>/origin.git` holding main, and
/// the repository's HEAD detached so main is checked out nowhere.
fn setup() -> Setup {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    let tmp_repo = TestRepo::new().unwrap();
    let tmp = tmp_repo.path.canonicalize().unwrap();
    cli::write_config(&root.join(".git"), &Config {
        origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
        zones: BTreeMap::from([
            ("a".into(), ZoneConfig { paths: vec!["src/a".into()] }),
            ("b".into(), ZoneConfig { paths: vec!["src/b".into()] }),
        ]),
        root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
        floor_bytes: 0, formatter: None,
    }).unwrap();
    write(&root, "src/a/x", X);
    write(&root, "src/b/y", "y\n");
    write(&root, "Cargo.toml", "[workspace]\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "first"]);
    git(&root, &["branch", "integration"]);
    git(&root, &["checkout", "-q", "-b", "zone/a"]);
    write(&root, "src/a/x", X_ZONE);
    git(&root, &["commit", "-qam", "edit src/a/x"]);
    git(&root, &["checkout", "-q", "--detach", "main"]);
    let bare = tmp.join("origin.git");
    git(&tmp, &["init", "-q", "--bare", "--initial-branch=main", bare.to_str().unwrap()]);
    git(&root, &["remote", "add", "origin", bare.to_str().unwrap()]);
    git(&root, &["push", "-q", "origin", "refs/heads/main:refs/heads/main"]);
    Setup { _repo: repo, _tmp: tmp_repo, root, tmp, bare }
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A setup git command in `dir`, asserted to succeed; its stdout, trimmed.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git").arg("-C").arg(dir).args(args).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).output().unwrap();
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim_end().to_string()
}

fn tip(repo: &Path, refname: &str) -> String {
    git(repo, &["rev-parse", "--verify", refname])
}

/// Whether `refname` exists in `repo`.
fn exists(repo: &Path, refname: &str) -> bool {
    Command::new("git").arg("-C").arg(repo).args(["rev-parse", "-q", "--verify", refname]).env_clear()
        .envs(GIT_ENV).env("PATH", std::env::var_os("PATH").unwrap()).output().unwrap().status.success()
}

/// Zone a squashed through land::squash, then landed through land::land, both under the land lock after recover,
/// as the cli does; the landing is asserted to have fast-forwarded integration to the squash.
fn squash_and_land(root: &Path) -> Squash {
    let mut git = Git::new(root);
    git.env = GIT_ENV.iter().map(|(name, value)| (name.into(), value.into())).collect();
    let common = git.common_dir().unwrap();
    let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() };
    oplog::with_lock(&git, &common, &LockName::Land, "publish-setup", &KillPidProbe, || {
        oplog::recover(&git, &common).unwrap();
        let SquashAnswer::Made(squash) = land::squash(&git, &refs, "a", "publish-setup").unwrap() else {
            panic!("setup squash was not made");
        };
        let landed = land::land(&git, &refs, &squash, "publish-setup").unwrap();
        assert_eq!(landed, LandAnswer::Landed { integration: squash.commit.clone() });
        Ok::<_, std::io::Error>(squash)
    }).unwrap()
}

/// `mortar <args>` run in `root`, git's config isolated.
fn mortar(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(root).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "publish-test").stdin(Stdio::null()).args(args).output().unwrap()
}

fn publish(root: &Path) -> Output {
    mortar(root, &["--actor", "accept", "--json", "publish"])
}

#[test]
fn publish_after_a_landing_moves_the_remote_then_main_and_removes_the_squash_ref() {
    let s = setup();
    let squash = squash_and_land(&s.root);
    let report = check(publish(&s.root), "publish", 0, "published");
    assert_eq!(report.text("origin"), squash.commit.as_str());
    assert_eq!(tip(&s.bare, "refs/heads/main"), squash.commit.as_str());
    assert_eq!(tip(&s.root, "refs/heads/main"), squash.commit.as_str());
    assert!(!exists(&s.root, SQUASH_A));
}

#[test]
fn publish_with_no_landing_has_nothing_to_publish_and_leaves_the_remote() {
    let s = setup();
    let remote = tip(&s.bare, "refs/heads/main");
    check(publish(&s.root), "publish", 0, "nothing-to-publish");
    assert_eq!(tip(&s.bare, "refs/heads/main"), remote);
    assert_eq!(tip(&s.root, "refs/heads/main"), remote);
}

#[test]
fn publish_after_someone_else_moved_the_remote_is_origin_moved_onto_their_commit() {
    let s = setup();
    let folder = s.tmp.join("int");
    git(&s.root, &["worktree", "add", "-q", folder.to_str().unwrap(), "integration"]);
    let main = tip(&s.root, "refs/heads/main");
    let squash = squash_and_land(&s.root);
    git(&folder, &["read-tree", "-m", "-u", &main, squash.commit.as_str()]);
    assert_eq!(fs::read_to_string(folder.join("src/a/x")).unwrap(), X_ZONE);
    let other = s.tmp.join("other");
    git(&s.tmp, &["clone", "-q", s.bare.to_str().unwrap(), other.to_str().unwrap()]);
    write(&other, "src/b/other", OTHER);
    git(&other, &["add", "-A"]);
    git(&other, &["commit", "-qm", "another clone's work"]);
    git(&other, &["push", "-q", "origin", "main"]);
    let found = tip(&other, "HEAD");
    let report = check(publish(&s.root), "publish", 1, "origin-moved");
    assert_eq!(report.text("found"), found);
    assert_eq!(report.text("integration"), found);
    assert_eq!(report.text("folder"), folder.to_str().unwrap());
    assert_eq!(tip(&s.root, "refs/heads/integration"), found);
    assert_eq!(tip(&s.root, "refs/heads/main"), found);
    assert_eq!(tip(&s.bare, "refs/heads/main"), found);
    assert_eq!(tip(&s.root, SQUASH_A), squash.commit.as_str());
    assert_eq!(git(&folder, &["status", "--porcelain"]), "");
    assert_eq!(fs::read_to_string(folder.join("src/b/other")).unwrap(), OTHER);
    assert_eq!(fs::read_to_string(folder.join("src/a/x")).unwrap(), X);
}

#[test]
fn publish_with_an_extra_word_is_usage_naming_publishs_usage() {
    let s = setup();
    let squash = squash_and_land(&s.root);
    let remote = tip(&s.bare, "refs/heads/main");
    let report = check(mortar(&s.root, &["--actor", "accept", "--json", "publish", "now"]), "publish", 4, "usage");
    assert_eq!(report.text("next"), USAGE);
    assert_eq!(tip(&s.bare, "refs/heads/main"), remote);
    assert_ne!(remote, squash.commit.as_str());
}
