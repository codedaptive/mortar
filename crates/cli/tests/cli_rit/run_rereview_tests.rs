//! `mortar rereview <reviewed> <new>`: two squashes of one change answer rereviewed with both ids and a diff that
//! names the file the second squash added and not the line both carry; squashes of two changes are refused as
//! not-same-change (exit 1); one word, three words and a word that is not a full commit id are usage errors (exit 4)
//! with rereview's usage line as next.
//! RIT (S-135, S-136): drives the mortar binary against real git and the crates below cli; owned by Nagatha from
//! landing. The squashes are made in setup through land::squash under the land lock.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119):
//! - without a config: exit 1, next mortar init (the shell's load_config);
//! - RereviewError::Git and RereviewError::Trailers texts (git or the trailers must fail);
//! - RereviewError::NoParent: a root commit;
//! - ACCEPT-1 checks 7, 9, 11 and 12 through these verbs on two zones.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::{Config, ZoneConfig};
use gitio::Git;
use crate::json::check;
use land::{LandRefs, Squash, SquashAnswer};
use oplog::{KillPidProbe, LockName};
use testkit::TestRepo;

const USAGE: &str = "mortar rereview <reviewed commit> <new commit> --actor <name> [--key <operation key>] [--json]";
const X: &str = "x1\nx2\nx3\nx4\nx5\nx6\nx7\nx8\n";
const X_EDIT_LINE: &str = "zone edit of x1";
const X_ZONE: &str = "zone edit of x1\nx2\nx3\nx4\nx5\nx6\nx7\nx8\n";

const GIT_ENV: [(&str, &str); 6] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_AUTHOR_NAME", "Rereview Verb"),
    ("GIT_AUTHOR_EMAIL", "rereview-verb@example.invalid"),
    ("GIT_COMMITTER_NAME", "Rereview Verb"),
    ("GIT_COMMITTER_EMAIL", "rereview-verb@example.invalid"),
];

/// A throwaway repository (removed on drop) with zones a (src/a) and b (src/b), main's first commit holding
/// src/a/x, src/b/y and Cargo.toml, integration at that commit, and its canonical root.
fn repo() -> (TestRepo, PathBuf) {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
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
    git(&root, &["branch", "zone/a"]);
    git(&root, &["branch", "zone/b"]);
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

/// One commit on `branch` writing `text` to `path`; main is checked out again after.
fn commit_on(root: &Path, branch: &str, path: &str, text: &str) {
    git(root, &["checkout", "-q", branch]);
    write(root, path, text);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", &format!("edit {path}")]);
    git(root, &["checkout", "-q", "main"]);
}

/// A squash of `zone` made in setup through land::squash, under the land lock after recover, as the cli does.
fn squash_in_setup(root: &Path, zone: &str) -> Squash {
    let mut git = Git::new(root);
    git.env = GIT_ENV.iter().map(|(name, value)| (name.into(), value.into())).collect();
    let common = git.common_dir().unwrap();
    let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() };
    let answer = oplog::with_lock(&git, &common, &LockName::Land, "rereview-setup", &KillPidProbe, || {
        oplog::recover(&git, &common).unwrap();
        Ok::<_, std::io::Error>(land::squash(&git, &refs, zone, "rereview-setup").unwrap())
    }).unwrap();
    match answer {
        SquashAnswer::Made(squash) => squash,
        other => panic!("setup squash of {zone}: {other:?}"),
    }
}

/// `mortar <args>` run in `root`, git's config isolated.
fn mortar(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(root).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "rereview-test").stdin(Stdio::null()).args(args).output().unwrap()
}

/// Two squashes of zone a's one change: the first after an edit of src/a/x, the second after a further commit
/// adding src/a/z.
fn two_squashes_of_one_change(root: &Path) -> (Squash, Squash) {
    commit_on(root, "zone/a", "src/a/x", X_ZONE);
    let reviewed = squash_in_setup(root, "a");
    commit_on(root, "zone/a", "src/a/z", "z content\n");
    let new = squash_in_setup(root, "a");
    assert_eq!(new.change_id, reviewed.change_id, "setup: a resquash keeps the change ID");
    assert_ne!(new.commit, reviewed.commit);
    (reviewed, new)
}

#[test]
fn rereview_of_two_squashes_of_one_change_shows_only_what_changed() {
    let (_repo, root) = repo();
    let (reviewed, new) = two_squashes_of_one_change(&root);
    let output = mortar(&root, &["--actor", "accept", "--json", "rereview", reviewed.commit.as_str(), new.commit.as_str()]);
    let report = check(output, "rereview", 0, "rereviewed");
    assert_eq!(report.text("reviewed"), reviewed.commit.as_str());
    assert_eq!(report.text("new"), new.commit.as_str());
    let lines = report.lines();
    assert!(lines.contains("src/a/z"), "{lines}");
    assert!(!lines.contains(X_EDIT_LINE), "{lines}");
}

#[test]
fn rereview_of_two_different_changes_is_not_same_change() {
    let (_repo, root) = repo();
    commit_on(&root, "zone/a", "src/a/x", X_ZONE);
    commit_on(&root, "zone/b", "src/b/y", "zone edit of y\n");
    let a = squash_in_setup(&root, "a");
    let b = squash_in_setup(&root, "b");
    let output = mortar(&root, &["--actor", "accept", "--json", "rereview", a.commit.as_str(), b.commit.as_str()]);
    check(output, "rereview", 1, "not-same-change");
}

#[test]
fn rereview_usage_errors_name_rereviews_usage() {
    let (_repo, root) = repo();
    let (reviewed, new) = two_squashes_of_one_change(&root);
    let (reviewed, new) = (reviewed.commit.as_str(), new.commit.as_str());
    for args in [
        &["--actor", "accept", "--json", "rereview", reviewed][..],
        &["--actor", "accept", "--json", "rereview", reviewed, new, reviewed],
        &["--actor", "accept", "--json", "rereview", "HEAD", new],
    ] {
        let report = check(mortar(&root, args), "rereview", 4, "usage");
        assert_eq!(report.text("next"), USAGE, "{args:?}");
    }
}
