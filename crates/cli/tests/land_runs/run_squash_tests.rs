//! `mortar squash <zone>`: a zone one commit ahead of main is squashed onto main's tip at refs/mortar/squash/<zone>
//! with main, integration and the zone branch unmoved; a repeat is already-squashed with the same commit and change
//! ID; a zone at main has nothing to squash; a zone that conflicts with main exits 1 naming the file; a configured
//! zone with no zone branch is refused (exit 3) naming the branch; no zone word, two words, an unknown zone and no
//! --actor are usage errors (exit 2) with squash's usage line as next and no squash ref written.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.
//! Tests to write (not written here):
//! - the land lock held by a live pid: exit 4 (cli with oplog's locks);
//! - without a config: exit 2, next mortar init (the shell's load_config);
//! - each land error variant's text that needs git or the op log to fail: SquashError Git, Oplog, ChangeId,
//!   Trailers, OpFailed and OpRepeated;
//! - acceptance step 1's checks 7, 9, 11 and 12 through these verbs on two zones.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::support::{Config, ZoneConfig, write_fixture_config};
use crate::json::check;
use testkit::TestRepo;

const USAGE: &str = "mr squash <zone> [--message <text>] --actor <name> [--key <operation key>] [--json]";
const SQUASH_A: &str = "refs/mortar/squash/a";
const X: &str = "x1\nx2\nx3\nx4\nx5\nx6\n";
const X_ZONE: &str = "zone edit of x1\nx2\nx3\nx4\nx5\nx6\n";
const X_MAIN: &str = "main edit of x1\nx2\nx3\nx4\nx5\nx6\n";

const GIT_ENV: [(&str, &str); 6] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_AUTHOR_NAME", "Squash Verb"),
    ("GIT_AUTHOR_EMAIL", "squash-verb@example.invalid"),
    ("GIT_COMMITTER_NAME", "Squash Verb"),
    ("GIT_COMMITTER_EMAIL", "squash-verb@example.invalid"),
];

/// A throwaway repository (removed on drop) with zones a (src/a) and b (src/b), main's first commit holding
/// src/a/x, src/b/y and Cargo.toml, integration at that commit, and its canonical root.
fn repo() -> (TestRepo, PathBuf) {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    write_fixture_config(&root, &Config {
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
    (repo, root)
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

/// The commit at `refname`, or None when the ref does not exist.
fn tip(root: &Path, refname: &str) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(root).args(["rev-parse", "-q", "--verify", refname])
        .env_clear().envs(GIT_ENV).env("PATH", std::env::var_os("PATH").unwrap()).output().unwrap();
    output.status.success().then(|| String::from_utf8(output.stdout).unwrap().trim_end().to_string())
}

/// One commit on `branch` writing `text` to `path`; main is checked out again after. The new tip.
fn commit_on(root: &Path, branch: &str, path: &str, text: &str) -> String {
    git(root, &["checkout", "-q", branch]);
    write(root, path, text);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", &format!("edit {path}")]);
    let tip = git(root, &["rev-parse", "HEAD"]);
    git(root, &["checkout", "-q", "main"]);
    tip
}

/// zone/a made at main, then one commit on it editing the first line of src/a/x.
fn zone_a_ahead(root: &Path) -> String {
    git(root, &["branch", "zone/a"]);
    commit_on(root, "zone/a", "src/a/x", X_ZONE)
}

/// `mortar <args>` run in `root`, git's config isolated.
fn mortar(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(root).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "squash-test").stdin(Stdio::null()).args(args).output().unwrap()
}

fn squash(root: &Path, zone: &str) -> Output {
    mortar(root, &["--actor", "accept", "--json", "squash", zone])
}

fn squash_refs(root: &Path) -> String {
    git(root, &["for-each-ref", "refs/mortar/squash"])
}

#[test]
fn squash_of_a_zone_one_commit_ahead_is_squashed_onto_mains_tip() {
    let (_repo, root) = repo();
    let zone = zone_a_ahead(&root);
    let main = tip(&root, "refs/heads/main").unwrap();
    let report = check(squash(&root, "a"), "squash", 0, "squashed");
    assert_eq!(report.text("zone"), "a");
    let change_id = report.text("change_id");
    assert!(!change_id.is_empty());
    let commit = report.text("commit");
    assert_eq!(report.text("source"), zone);
    assert_eq!(report.strings("requests"), Vec::<String>::new());
    assert_eq!(tip(&root, SQUASH_A).as_deref(), Some(commit));
    assert_eq!(git(&root, &["log", "-1", "--format=%P", commit]), main);
    assert_eq!(git(&root, &["show", &format!("{commit}:src/a/x")]), X_ZONE.trim_end());
    assert!(git(&root, &["log", "-1", "--format=%B", commit]).contains(&format!("Mortar-Change-Id: {change_id}")));
    assert_eq!(tip(&root, "refs/heads/main").unwrap(), main);
    assert_eq!(tip(&root, "refs/heads/integration").unwrap(), main);
    assert_eq!(tip(&root, "refs/heads/zone/a").unwrap(), zone);
}

#[test]
fn squash_repeated_is_already_squashed_with_the_same_commit_and_change_id() {
    let (_repo, root) = repo();
    zone_a_ahead(&root);
    let first = check(squash(&root, "a"), "squash", 0, "squashed");
    let before = tip(&root, SQUASH_A).unwrap();
    let again = check(squash(&root, "a"), "squash", 0, "already-squashed");
    assert_eq!(again.text("commit"), first.text("commit"));
    assert_eq!(again.text("change_id"), first.text("change_id"));
    assert_eq!(tip(&root, SQUASH_A).unwrap(), before);
}

#[test]
fn squash_of_a_zone_at_main_has_nothing_to_squash() {
    let (_repo, root) = repo();
    git(&root, &["branch", "zone/a"]);
    check(squash(&root, "a"), "squash", 0, "nothing-to-squash");
    assert_eq!(tip(&root, SQUASH_A), None);
}

#[test]
fn squash_conflicting_with_main_exits_2_naming_the_file() {
    let (_repo, root) = repo();
    zone_a_ahead(&root);
    commit_on(&root, "main", "src/a/x", X_MAIN);
    let report = check(squash(&root, "a"), "squash", 1, "conflict");
    assert!(report.strings("files").contains(&"src/a/x".to_string()), "{:?}", report.strings("files"));
    assert_eq!(tip(&root, SQUASH_A), None);
}

#[test]
fn squash_of_a_zone_with_no_zone_branch_is_refused_naming_the_branch() {
    let (_repo, root) = repo();
    let report = check(squash(&root, "b"), "squash", 3, "refused");
    assert!(report.strings("lines").iter().any(|line| line.contains("refs/heads/zone/b")), "{}", report.lines());
    assert_eq!(squash_refs(&root), "");
}

#[test]
fn squash_usage_errors_name_squashs_usage_and_write_no_squash_ref() {
    let (_repo, root) = repo();
    zone_a_ahead(&root);
    for args in [
        &["--actor", "accept", "--json", "squash"][..],
        &["--actor", "accept", "--json", "squash", "a", "b"],
        &["--actor", "accept", "--json", "squash", "c"],
        &["--json", "squash", "a"],
    ] {
        let report = check(mortar(&root, args), "squash", 2, "usage");
        assert_eq!(report.text("next"), USAGE, "{args:?}");
        assert_eq!(squash_refs(&root), "", "{args:?}");
    }
}
