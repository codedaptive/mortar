//! `mortar back-out`: after a fast-forward landing, integration goes back to main's tip, the integration worktree's
//! files follow it and the squash ref stays; a repeat, and a repository where nothing landed, have nothing to back
//! out; an extra word is a usage error (exit 4) with back-out's usage line as next and integration unmoved.
//! RIT (S-135, S-136): drives the mortar binary against real git and the crates below cli; owned by Nagatha from
//! landing. The squash and the landing are made in setup through land::squash and land::land under the land lock.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119):
//! - the land lock held by a live pid: exit 3 (cli with oplog's locks);
//! - without a config: exit 1, next mortar init (the shell's load_config);
//! - each LandError variant's text that needs git or the op log to fail;
//! - ACCEPT-1 checks 7, 9, 11 and 12 through these verbs on two zones;
//! - back-out with --key: the key names the lock holder only (S-132).

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

const USAGE: &str = "mortar back-out --actor <name> [--key <operation key>] [--json]";
const SQUASH_A: &str = "refs/mortar/squash/a";
const X: &str = "x1\nx2\nx3\n";
const X_ZONE: &str = "zone edit of x1\nx2\nx3\n";

const GIT_ENV: [(&str, &str); 6] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_AUTHOR_NAME", "Back Out Verb"),
    ("GIT_AUTHOR_EMAIL", "back-out-verb@example.invalid"),
    ("GIT_COMMITTER_NAME", "Back Out Verb"),
    ("GIT_COMMITTER_EMAIL", "back-out-verb@example.invalid"),
];

/// A throwaway repository (removed on drop) with zones a (src/a) and b (src/b), main's first commit holding
/// src/a/x, src/b/y and Cargo.toml, integration at that commit, zone/a one commit ahead editing src/a/x, and its
/// canonical root.
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
    git(&root, &["checkout", "-q", "-b", "zone/a"]);
    write(&root, "src/a/x", X_ZONE);
    git(&root, &["commit", "-qam", "edit src/a/x"]);
    git(&root, &["checkout", "-q", "main"]);
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

fn tip(root: &Path, refname: &str) -> String {
    git(root, &["rev-parse", "--verify", refname])
}

/// The integration worktree, made at `<tmp>/int` (tmp a second throwaway directory); its canonical path.
fn integration_worktree(root: &Path, tmp: &TestRepo) -> PathBuf {
    let folder = tmp.path.canonicalize().unwrap().join("int");
    git(root, &["worktree", "add", "-q", folder.to_str().unwrap(), "integration"]);
    folder
}

/// Zone a squashed through land::squash, then landed through land::land, both under the land lock after recover,
/// as the cli does; the landing is asserted to have fast-forwarded integration to the squash.
fn squash_and_land(root: &Path) -> Squash {
    let mut git = Git::new(root);
    git.env = GIT_ENV.iter().map(|(name, value)| (name.into(), value.into())).collect();
    let common = git.common_dir().unwrap();
    let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() };
    oplog::with_lock(&git, &common, &LockName::Land, "back-out-setup", &KillPidProbe, || {
        oplog::recover(&git, &common).unwrap();
        let SquashAnswer::Made(squash) = land::squash(&git, &refs, "a", "back-out-setup").unwrap() else {
            panic!("setup squash was not made");
        };
        let landed = land::land(&git, &refs, &squash, "back-out-setup").unwrap();
        assert_eq!(landed, LandAnswer::Landed { integration: squash.commit.clone() });
        Ok::<_, std::io::Error>(squash)
    }).unwrap()
}

/// `mortar <args>` run in `root`, git's config isolated.
fn mortar(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(root).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "back-out-test").stdin(Stdio::null()).args(args).output().unwrap()
}

fn back_out(root: &Path) -> Output {
    mortar(root, &["--actor", "accept", "--json", "back-out"])
}

#[test]
fn back_out_after_a_fast_forward_landing_puts_integration_and_its_folder_back_on_main() {
    let (_repo, root) = repo();
    let tmp = TestRepo::new().unwrap();
    let folder = integration_worktree(&root, &tmp);
    let squash = squash_and_land(&root);
    let main = tip(&root, "refs/heads/main");
    git(&folder, &["read-tree", "-m", "-u", &main, squash.commit.as_str()]);
    assert_eq!(fs::read_to_string(folder.join("src/a/x")).unwrap(), X_ZONE);
    let report = check(back_out(&root), "back-out", 0, "backed-out");
    assert_eq!(report.text("change_id"), squash.change_id.as_str());
    assert_eq!(report.text("integration"), main);
    assert_eq!(report.text("folder"), folder.to_str().unwrap());
    assert_eq!(tip(&root, "refs/heads/integration"), main);
    assert_eq!(fs::read_to_string(folder.join("src/a/x")).unwrap(), X);
    assert_eq!(git(&folder, &["status", "--porcelain"]), "");
    assert_eq!(tip(&root, SQUASH_A), squash.commit.as_str());
}

#[test]
fn back_out_repeated_has_nothing_to_back_out() {
    let (_repo, root) = repo();
    let tmp = TestRepo::new().unwrap();
    integration_worktree(&root, &tmp);
    squash_and_land(&root);
    check(back_out(&root), "back-out", 0, "backed-out");
    check(back_out(&root), "back-out", 0, "nothing-to-back-out");
    assert_eq!(tip(&root, "refs/heads/integration"), tip(&root, "refs/heads/main"));
}

#[test]
fn back_out_with_nothing_landed_and_no_worktree_has_nothing_to_back_out() {
    let (_repo, root) = repo();
    let main = tip(&root, "refs/heads/main");
    let report = check(back_out(&root), "back-out", 0, "nothing-to-back-out");
    assert!(report.is_null("folder"));
    assert_eq!(tip(&root, "refs/heads/integration"), main);
}

#[test]
fn back_out_with_an_extra_word_is_usage_and_leaves_integration() {
    let (_repo, root) = repo();
    let squash = squash_and_land(&root);
    let report = check(mortar(&root, &["--actor", "accept", "--json", "back-out", "now"]), "back-out", 4, "usage");
    assert_eq!(report.text("next"), USAGE);
    assert_eq!(tip(&root, "refs/heads/integration"), squash.commit.as_str());
}
