//! `mortar guard install`: a fresh repository gets guard.json and both hooks, executable, each the guard's one-line
//! script; a repeat says already-installed and changes no byte; a foreign pre-push is refused (exit 3) with nothing
//! written; no --actor, no sub-verb and an unknown sub-verb are usage errors (exit 2) with guard's usage line as next;
//! from a linked worktree the hooks land in the main repository's hooks folder; a relative core.hooksPath is refused
//! (exit 3) with nothing written.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.
//! Tests to write (not written here):
//! - the land lock held by a live pid: exit 4 (cli with oplog's locks);
//! - without a config: exit 3, next mortar init (the shell's load_config);
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

const USAGE: &str = "mr guard (install | status) --actor <name> [--key <operation key>] [--json]";
const CONFIG: &str = "{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}\n";
const REFERENCE_TRANSACTION: &str = "#!/bin/sh\nexec mortar guard-hook reference-transaction \"$@\"\n";
const PRE_PUSH: &str = "#!/bin/sh\nexec mortar guard-hook pre-push \"$@\"\n";
const FOREIGN: &str = "#!/bin/sh\nexit 0\n";
const INSTALLED_NEXT: &str = "nothing to do: git now refuses moves of integration and main that Mortar did not make";

const GIT_ENV: [(&str, &str); 6] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_AUTHOR_NAME", "Guard Install"),
    ("GIT_AUTHOR_EMAIL", "guard-install@example.invalid"),
    ("GIT_COMMITTER_NAME", "Guard Install"),
    ("GIT_COMMITTER_EMAIL", "guard-install@example.invalid"),
];

/// A throwaway repository (removed on drop) with the config main / integration / origin, and its canonical root,
/// the form git prints its absolute paths in.
fn repo() -> (TestRepo, PathBuf) {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    write_fixture_config(&root, &Config {
        origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
        zones: BTreeMap::from([("core".into(), ZoneConfig { paths: vec!["src/a".into()] })]),
        root_files: vec!["Cargo.toml".into()], build_dirs: vec![], cap_bytes: snapshot::DEFAULT_SIZE_CAP,
        floor_bytes: 0, formatter: None,
    }).unwrap();
    (repo, root)
}

/// A setup git command in `dir`, asserted to succeed.
fn run_git(dir: &Path, args: &[&str]) {
    let output = Command::new("git").arg("-C").arg(dir).args(args).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).output().unwrap();
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
}

/// `mortar <args>` run in `cwd`, git's config isolated.
fn mortar(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(cwd).env_clear().envs(GIT_ENV)
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("CODETEAM_SESSION", "guard-install-test").stdin(Stdio::null()).args(args).output().unwrap()
}

fn install(root: &Path, cwd: &Path) -> Output {
    mortar(root, cwd, &["--actor", "harness", "--json", "guard", "install"])
}

/// The permission bits of `path`: land's guard_install sets each hook's mode to 0o755.
fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn shown(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

/// Both hooks in `hooks`, mode 0o755, each byte-for-byte the guard's script for its name.
fn assert_hooks_written(hooks: &Path) {
    assert_eq!(fs::read_to_string(hooks.join("reference-transaction")).unwrap(), REFERENCE_TRANSACTION);
    assert_eq!(fs::read_to_string(hooks.join("pre-push")).unwrap(), PRE_PUSH);
    assert_eq!(mode(&hooks.join("reference-transaction")), 0o755);
    assert_eq!(mode(&hooks.join("pre-push")), 0o755);
}

fn snapshot_files(root: &Path) -> Vec<Vec<u8>> {
    ["mortar/guard.json", "hooks/reference-transaction", "hooks/pre-push"]
        .map(|name| fs::read(root.join(".git").join(name)).unwrap()).to_vec()
}

#[test]
fn guard_install_fresh_writes_guard_json_and_both_hooks() {
    let (_repo, root) = repo();
    let report = check(install(&root, &root), "guard", 0, "installed");
    let hooks = root.join(".git/hooks");
    assert_hooks_written(&hooks);
    assert_eq!(fs::read_to_string(root.join(".git/mortar/guard.json")).unwrap(), CONFIG);
    assert_eq!(report.text("next"), INSTALLED_NEXT);
    assert!(report.lines().contains(&shown(&hooks)), "{}", report.lines());
    assert_eq!(report.text("hooks_dir"), shown(&hooks));
    assert_eq!(report.text("reference_transaction"), "mortar");
    assert_eq!(report.text("pre_push"), "mortar");
    assert_eq!(report.text("origin_branch"), "main");
    assert_eq!(report.text("integration_branch"), "integration");
    assert_eq!(report.text("remote"), "origin");
}

#[test]
fn guard_install_repeat_is_already_installed_and_changes_no_byte() {
    let (_repo, root) = repo();
    check(install(&root, &root), "guard", 0, "installed");
    let before = snapshot_files(&root);
    let report = check(install(&root, &root), "guard", 0, "already-installed");
    assert_eq!(snapshot_files(&root), before);
    assert_eq!(mode(&root.join(".git/hooks/reference-transaction")), 0o755);
    assert_eq!(mode(&root.join(".git/hooks/pre-push")), 0o755);
    assert_eq!(report.text("next"), INSTALLED_NEXT);
    assert!(report.lines().contains("already installed"), "{}", report.lines());
    assert_eq!(report.text("reference_transaction"), "mortar");
    assert_eq!(report.text("pre_push"), "mortar");
}

#[test]
fn guard_install_refuses_a_foreign_pre_push_and_writes_nothing() {
    let (_repo, root) = repo();
    let foreign = root.join(".git/hooks/pre-push");
    fs::write(&foreign, FOREIGN).unwrap();
    fs::set_permissions(&foreign, fs::Permissions::from_mode(0o755)).unwrap();
    let report = check(install(&root, &root), "guard", 3, "foreign-hook");
    let path = shown(&foreign);
    assert!(report.strings("lines").contains(&format!(
        "{path} is a hook Mortar did not write; Mortar never overwrites one")), "{}", report.lines());
    assert!(report.strings("lines").contains(&"nothing was written".to_string()), "{}", report.lines());
    assert_eq!(report.text("owner"), "the repository's own hooks");
    assert_eq!(report.text("next"), format!(
        "move {path} aside or fold it into your own hook by hand, then mr guard install --actor <name>"));
    assert_eq!(fs::read_to_string(&foreign).unwrap(), FOREIGN);
    assert!(!root.join(".git/hooks/reference-transaction").exists());
    assert!(!root.join(".git/mortar/guard.json").exists());
}

#[test]
fn guard_install_without_actor_is_usage_naming_guards_usage() {
    let (_repo, root) = repo();
    let report = check(mortar(&root, &root, &["--json", "guard", "install"]), "guard", 2, "usage");
    assert_eq!(report.strings("lines")[0], "--actor <name> is required");
    assert_eq!(report.text("next"), USAGE);
    assert!(!root.join(".git/mortar/guard.json").exists());
    assert!(!root.join(".git/hooks/pre-push").exists());
}

#[test]
fn guard_install_with_no_sub_verb_is_usage_naming_guards_usage() {
    let (_repo, root) = repo();
    let report = check(mortar(&root, &root, &["--actor", "harness", "--json", "guard"]), "guard", 2, "usage");
    assert_eq!(report.strings("lines")[0], "guard needs a sub-verb: install or status");
    assert_eq!(report.text("next"), USAGE);
}

#[test]
fn guard_install_with_an_unknown_sub_verb_is_usage_naming_guards_usage() {
    let (_repo, root) = repo();
    let output = mortar(&root, &root, &["--actor", "harness", "--json", "guard", "frobnicate"]);
    let report = check(output, "guard", 2, "usage");
    assert_eq!(report.strings("lines")[0], "unknown guard sub-verb \"frobnicate\"");
    assert_eq!(report.text("next"), USAGE);
    assert!(!root.join(".git/hooks/pre-push").exists());
}

#[test]
fn guard_install_from_a_linked_worktree_puts_the_hooks_in_the_main_repository() {
    let (_repo, root) = repo();
    run_git(&root, &["commit", "--allow-empty", "-qm", "first"]);
    let linked = root.join("linked");
    run_git(&root, &["worktree", "add", "-q", "-b", "side", linked.to_str().unwrap()]);
    let report = check(install(&root, &linked), "guard", 0, "installed");
    let hooks = root.join(".git/hooks");
    assert_hooks_written(&hooks);
    assert_eq!(fs::read_to_string(root.join(".git/mortar/guard.json")).unwrap(), CONFIG);
    assert_eq!(report.text("hooks_dir"), shown(&hooks));
}

#[test]
fn guard_install_refuses_a_relative_core_hooks_path() {
    let (_repo, root) = repo();
    run_git(&root, &["config", "core.hooksPath", "myhooks"]);
    let report = check(install(&root, &root), "guard", 3, "refused");
    assert_eq!(
        report.strings("lines")[0],
        "core.hooksPath is \"myhooks\", not an absolute path: git resolves a relative hooks path in each folder, so \
         one install cannot guard them all"
    );
    assert_eq!(
        report.text("next"),
        "unset core.hooksPath or set it to an absolute path, then mr guard install --actor <name>"
    );
    assert!(!root.join("myhooks").exists());
    assert!(!root.join(".git/hooks/pre-push").exists());
    assert!(!root.join(".git/mortar/guard.json").exists());
}
