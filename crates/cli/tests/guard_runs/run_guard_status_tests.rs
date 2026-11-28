//! `mortar guard status`: an untouched repository is not-installed and nothing is written; after `mortar guard
//! install` it is installed with the three branches in its fields; a foreign reference-transaction exits 3 naming
//! its path; a guard.json that is not a guard config exits 2 bad-config naming the file; a guard.json for other branches
//! than the config is not-installed naming both.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.
//! Tests to write (not written here):
//! - the land lock held by a live pid: exit 4 (cli with oplog's locks);
//! - without a config: exit 2, next mortar init (the shell's load_config);
//! - GuardError::Git: the cli's own words, never gitio's Display text;
//! - after `mortar guard install`, git update-ref, commit and push against integration and origin are refused
//!   through the installed hook and Mortar's own moves pass (acceptance step 2's checks 1-9; land's guard_decide);
//! - guard_fixture.rs to install through `mortar guard install` instead of land::guard_install.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::support::{Config, ZoneConfig, write_fixture_config};
use crate::json::check;
use testkit::TestRepo;

const FOREIGN: &str = "#!/bin/sh\nexit 0\n";
const INSTALL_NEXT: &str = "mr guard install --actor <name>";

const GIT_ENV: [(&str, &str); 2] = [("GIT_CONFIG_GLOBAL", "/dev/null"), ("GIT_CONFIG_NOSYSTEM", "1")];

/// The config naming `origin` and `integration` as its branches and origin as its remote.
fn write_config(root: &Path, origin: &str, integration: &str) {
    write_fixture_config(root, &Config {
        origin_branch: origin.into(), integration_branch: integration.into(), remote: "origin".into(),
        zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src/a".into()] })]),
        root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
        floor_bytes: 0, formatter: None,
    }).unwrap();
}

/// A throwaway repository (removed on drop) with the config main / integration / origin, and its canonical root,
/// the form git prints its absolute paths in.
fn repo() -> (TestRepo, PathBuf) {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    write_config(&root, "main", "integration");
    (repo, root)
}

/// `mortar --actor harness --json guard <sub>` run in `root`, git's config isolated.
fn guard(root: &Path, sub: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(root).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "guard-status-test").stdin(Stdio::null())
        .args(["--actor", "harness", "--json", "guard", sub]).output().unwrap()
}

fn shown(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

#[test]
fn guard_status_of_an_untouched_repository_is_not_installed_and_writes_nothing() {
    let (_repo, root) = repo();
    let report = check(guard(&root, "status"), "guard", 0, "not-installed");
    assert_eq!(report.text("next"), INSTALL_NEXT);
    let lines = report.lines();
    for missing in ["reference-transaction", "pre-push", "guard.json"] {
        assert!(lines.contains(missing), "{missing} not named: {lines}");
    }
    assert_eq!(report.text("hooks_dir"), shown(&root.join(".git/hooks")));
    assert_eq!(report.text("reference_transaction"), "missing");
    assert_eq!(report.text("pre_push"), "missing");
    assert!(report.is_null("origin_branch"));
    assert!(report.is_null("integration_branch"));
    assert!(report.is_null("remote"));
    assert!(!root.join(".git/hooks/reference-transaction").exists());
    assert!(!root.join(".git/hooks/pre-push").exists());
    assert!(!root.join(".git/mortar/guard.json").exists());
}

#[test]
fn guard_status_after_install_is_installed_naming_the_three_branches() {
    let (_repo, root) = repo();
    check(guard(&root, "install"), "guard", 0, "installed");
    let report = check(guard(&root, "status"), "guard", 0, "installed");
    let hooks = shown(&root.join(".git/hooks"));
    assert_eq!(report.text("next"),
        "nothing to do: git now refuses moves of integration and main that Mortar did not make");
    let lines = report.lines();
    for named in [hooks.as_str(), "refs/heads/integration", "refs/heads/main", "origin"] {
        assert!(lines.contains(named), "{named} not named: {lines}");
    }
    assert_eq!(report.text("hooks_dir"), hooks);
    assert_eq!(report.text("reference_transaction"), "mortar");
    assert_eq!(report.text("pre_push"), "mortar");
    assert_eq!(report.text("origin_branch"), "main");
    assert_eq!(report.text("integration_branch"), "integration");
    assert_eq!(report.text("remote"), "origin");
}

#[test]
fn guard_status_of_a_foreign_reference_transaction_is_refused_naming_its_path() {
    let (_repo, root) = repo();
    let foreign = root.join(".git/hooks/reference-transaction");
    fs::write(&foreign, FOREIGN).unwrap();
    fs::set_permissions(&foreign, fs::Permissions::from_mode(0o755)).unwrap();
    let report = check(guard(&root, "status"), "guard", 3, "foreign-hook");
    let path = shown(&foreign);
    assert!(report.strings("lines").contains(&format!(
        "{path} is a hook Mortar did not write; Mortar never overwrites one")), "{}", report.lines());
    assert_eq!(report.text("owner"), "the repository's own hooks");
    assert_eq!(report.text("next"), format!(
        "move {path} aside or fold it into your own hook by hand, then mr guard install --actor <name>"));
    assert_eq!(report.text("reference_transaction"), "foreign");
    assert_eq!(report.text("pre_push"), "missing");
    assert_eq!(fs::read_to_string(&foreign).unwrap(), FOREIGN);
    assert!(!root.join(".git/hooks/pre-push").exists());
}

#[test]
fn guard_status_of_a_bad_guard_config_is_usage_naming_the_file() {
    let (_repo, root) = repo();
    let config = root.join(".git/mortar/guard.json");
    fs::write(&config, "{").unwrap();
    let report = check(guard(&root, "status"), "guard", 2, "bad-config");
    let plain = Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(&root).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", &root).env("TMPDIR", &root)
        .env("CODETEAM_SESSION", "guard-status-test").stdin(Stdio::null())
        .args(["--actor", "harness", "guard", "status"]).output().unwrap();
    assert_eq!(plain.status.code(), Some(2), "{plain:?}");
    let text = String::from_utf8(plain.stdout).unwrap();
    assert!(text.contains(config.to_str().unwrap())); assert!(text.contains(INSTALL_NEXT));
    let named = format!("{} is not a Mortar guard config: ", shown(&config));
    assert!(report.strings("lines").iter().any(|line| line.starts_with(&named)), "{}", report.lines());
    assert_eq!(report.text("owner"), "the Mortar guard");
    assert_eq!(report.text("next"), INSTALL_NEXT);
    assert_eq!(fs::read_to_string(&config).unwrap(), "{");
}

#[test]
fn guard_status_of_a_guard_json_for_other_branches_is_not_installed_naming_both() {
    let (_repo, root) = repo();
    check(guard(&root, "install"), "guard", 0, "installed");
    write_config(&root, "release", "develop");
    let report = check(guard(&root, "status"), "guard", 0, "not-installed");
    assert_eq!(report.text("next"), INSTALL_NEXT);
    let lines = report.lines();
    for named in ["release", "develop", "main", "integration"] {
        assert!(lines.contains(named), "{named} not named: {lines}");
    }
    assert_eq!(report.text("reference_transaction"), "mortar");
    assert_eq!(report.text("pre_push"), "mortar");
    assert_eq!(report.text("origin_branch"), "main");
    assert_eq!(report.text("integration_branch"), "integration");
}
