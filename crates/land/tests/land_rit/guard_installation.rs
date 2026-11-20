//! Land -> Gitio RITs joining guard_install, guard_status and read_guard_config on disposable TestRepos.
//! guard_installation.rs holds thirteen real-repository guard cases with literal byte/state oracles.
//! check_hooks_path.rs covers the relative-path check's refusal in isolation, not its ordering with guard_install.
//! CLI run_guard_install owns that ordering.
//! Every setup and public-API Git call isolates global/system config; TestRepo removes its repository on drop.

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

use gitio::Git;
use land::{guard_install, guard_status, read_guard_config, GuardError, GuardState, HookState, LandRefs};
use testkit::TestRepo;

const CONFIG: &str = "{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}\n";
const REFERENCE_TRANSACTION: &str = "#!/bin/sh\nexec mortar guard-hook reference-transaction \"$@\"\n";
const PRE_PUSH: &str = "#!/bin/sh\nexec mortar guard-hook pre-push \"$@\"\n";
const FOREIGN: &str = "#!/bin/sh\nexit 0\n";

fn env() -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_AUTHOR_NAME", "Guard Zit"),
        ("GIT_AUTHOR_EMAIL", "guard-zit@example.com"),
        ("GIT_COMMITTER_NAME", "Guard Zit"),
        ("GIT_COMMITTER_EMAIL", "guard-zit@example.com"),
    ]
    .map(|(k, v)| (OsString::from(k), OsString::from(v)))
    .to_vec()
}

fn git(dir: &Path) -> Git {
    Git { program: PathBuf::from("git"), repo: dir.to_path_buf(), env: env() }
}

/// A setup git command in `dir`, asserted to succeed.
fn run_git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    command.arg("-C").arg(dir).args(args).envs(env());
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"] {
        command.env_remove(name);
    }
    let output = command.output().unwrap();
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
}

fn refs() -> LandRefs {
    LandRefs {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
    }
}

/// A fresh repository and its canonical path, the form git prints its absolute paths in.
fn repo() -> (TestRepo, PathBuf) {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    (repo, root)
}

fn hook_mode(path: &Path) -> u32 {
    // Include setuid, setgid and sticky bits as well as rwx bits; file type is not part of the installed permissions.
    fs::metadata(path).unwrap().permissions().mode() & 0o7777
}

fn installed(hooks_dir: PathBuf) -> GuardState {
    GuardState { refs: Some(refs()), hooks_dir, reference_transaction: HookState::Mortar, pre_push: HookState::Mortar }
}

#[test]
fn fresh_install_writes_config_and_hooks() {
    let (_repo, root) = repo();
    guard_install(&git(&root), &refs()).unwrap();
    assert_eq!(fs::read_to_string(root.join(".git/mortar/guard.json")).unwrap(), CONFIG);
    let hooks = root.join(".git/hooks");
    assert_eq!(fs::read_to_string(hooks.join("reference-transaction")).unwrap(), REFERENCE_TRANSACTION);
    assert_eq!(fs::read_to_string(hooks.join("pre-push")).unwrap(), PRE_PUSH);
    assert_eq!(hook_mode(&hooks.join("reference-transaction")), 0o755);
    assert_eq!(hook_mode(&hooks.join("pre-push")), 0o755);
    assert_eq!(guard_status(&git(&root)).unwrap(), installed(hooks));
}

#[test]
fn repeat_install_changes_nothing() {
    let (_repo, root) = repo();
    guard_install(&git(&root), &refs()).unwrap();
    guard_install(&git(&root), &refs()).unwrap();
    assert_eq!(fs::read_to_string(root.join(".git/mortar/guard.json")).unwrap(), CONFIG);
    assert_eq!(fs::read_to_string(root.join(".git/hooks/reference-transaction")).unwrap(), REFERENCE_TRANSACTION);
    assert_eq!(fs::read_to_string(root.join(".git/hooks/pre-push")).unwrap(), PRE_PUSH);
    assert_eq!(guard_status(&git(&root)).unwrap(), installed(root.join(".git/hooks")));
}

#[test]
fn a_foreign_pre_push_is_refused_and_nothing_written() {
    let (_repo, root) = repo();
    let foreign = root.join(".git/hooks/pre-push");
    fs::write(&foreign, FOREIGN).unwrap();
    let result = guard_install(&git(&root), &refs());
    assert!(matches!(&result, Err(GuardError::ForeignHook { path }) if *path == foreign), "{result:?}");
    assert_eq!(fs::read_to_string(&foreign).unwrap(), FOREIGN);
    assert!(!root.join(".git/hooks/reference-transaction").exists());
    assert!(!root.join(".git/mortar/guard.json").exists());
}

#[test]
fn a_foreign_reference_transaction_is_refused_and_nothing_written() {
    let (_repo, root) = repo();
    let foreign = root.join(".git/hooks/reference-transaction");
    fs::write(&foreign, FOREIGN).unwrap();
    let result = guard_install(&git(&root), &refs());
    assert!(matches!(&result, Err(GuardError::ForeignHook { path }) if *path == foreign), "{result:?}");
    assert_eq!(fs::read_to_string(&foreign).unwrap(), FOREIGN);
    assert!(!root.join(".git/hooks/pre-push").exists());
    assert!(!root.join(".git/mortar/guard.json").exists());
}

#[test]
fn install_from_a_linked_worktree_covers_the_main_repo() {
    let (_repo, root) = repo();
    run_git(&root, &["commit", "--allow-empty", "-m", "first"]);
    let linked = root.join("linked");
    run_git(&root, &["worktree", "add", "-b", "side", linked.to_str().unwrap()]);
    guard_install(&git(&linked), &refs()).unwrap();
    assert_eq!(fs::read_to_string(root.join(".git/hooks/pre-push")).unwrap(), PRE_PUSH);
    assert_eq!(guard_status(&git(&root)).unwrap(), installed(root.join(".git/hooks")));
}

#[test]
fn status_of_an_untouched_repo_and_of_a_foreign_hook() {
    let (_repo, root) = repo();
    let hooks_dir = root.join(".git/hooks");
    let untouched = GuardState {
        refs: None,
        hooks_dir: hooks_dir.clone(),
        reference_transaction: HookState::Missing,
        pre_push: HookState::Missing,
    };
    assert_eq!(guard_status(&git(&root)).unwrap(), untouched);
    fs::write(hooks_dir.join("pre-push"), FOREIGN).unwrap();
    let foreign = GuardState { pre_push: HookState::Foreign, ..untouched };
    assert_eq!(guard_status(&git(&root)).unwrap(), foreign);
}

#[test]
fn read_guard_config_missing_or_bad() {
    let (_repo, root) = repo();
    assert!(matches!(read_guard_config(&git(&root)), Ok(None)));
    let config = root.join(".git/mortar/guard.json");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "{\"remote\":\"origin\"}").unwrap();
    let result = read_guard_config(&git(&root));
    assert!(matches!(&result, Err(GuardError::BadConfig { path, .. }) if *path == config), "{result:?}");
}

/// A pre-push file holding `content` reads as Foreign, and install refuses it and writes nothing.
fn foreign_pre_push_is_reported_and_refused(content: &[u8]) {
    let (_repo, root) = repo();
    let hooks_dir = root.join(".git/hooks");
    let pre_push = hooks_dir.join("pre-push");
    fs::write(&pre_push, content).unwrap();
    let expected = GuardState {
        refs: None,
        hooks_dir,
        reference_transaction: HookState::Missing,
        pre_push: HookState::Foreign,
    };
    assert_eq!(guard_status(&git(&root)).unwrap(), expected);
    let result = guard_install(&git(&root), &refs());
    assert!(matches!(&result, Err(GuardError::ForeignHook { path }) if *path == pre_push), "{result:?}");
    assert_eq!(fs::read(&pre_push).unwrap(), content);
    assert!(!root.join(".git/hooks/reference-transaction").exists());
    assert!(!root.join(".git/mortar/guard.json").exists());
}

#[test]
fn foreign_pre_push_holding_the_reference_transaction_script() {
    foreign_pre_push_is_reported_and_refused(REFERENCE_TRANSACTION.as_bytes());
}

#[test]
fn foreign_pre_push_of_the_exact_script_plus_one_byte() {
    foreign_pre_push_is_reported_and_refused(format!("{PRE_PUSH}\n").as_bytes());
}

#[test]
fn foreign_pre_push_that_is_empty() {
    foreign_pre_push_is_reported_and_refused(b"");
}

#[test]
fn guard_install_preserves_victims_and_links_at_old_fixed_temp_names() {
    // Joins Land guard_install/write_replacing -> Gitio common_dir/hooks_path -> real file replacement.
    let (_repo, root) = repo();
    let hooks = root.join(".git/hooks");
    let mortar = root.join(".git/mortar");
    fs::create_dir_all(&hooks).unwrap();
    fs::create_dir_all(&mortar).unwrap();
    let planted = [
        (hooks.join("pre-push.mortar-tmp"), root.join("victim-pre-push")),
        (hooks.join("reference-transaction.mortar-tmp"), root.join("victim-reference-transaction")),
        (mortar.join("guard.json.mortar-tmp"), root.join("victim-guard-json")),
    ];
    for (link, victim) in &planted {
        fs::write(victim, b"untouched\n").unwrap();
        symlink(victim, link).unwrap();
    }

    guard_install(&git(&root), &refs()).unwrap();

    for (link, victim) in &planted {
        assert_eq!(fs::read(victim).unwrap(), b"untouched\n", "victim {} was overwritten", victim.display());
        assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink(), "{} was replaced", link.display());
        assert_eq!(fs::read_link(link).unwrap(), *victim);
    }
    assert_eq!(fs::read_to_string(mortar.join("guard.json")).unwrap(), CONFIG);
    assert_eq!(fs::read_to_string(hooks.join("pre-push")).unwrap(), PRE_PUSH);
    assert_eq!(fs::read_to_string(hooks.join("reference-transaction")).unwrap(), REFERENCE_TRANSACTION);
    assert_eq!(hook_mode(&hooks.join("pre-push")), 0o755);
    assert_eq!(hook_mode(&hooks.join("reference-transaction")), 0o755);
    for path in [hooks.join("pre-push"), hooks.join("reference-transaction"), mortar.join("guard.json")] {
        assert!(fs::symlink_metadata(&path).unwrap().file_type().is_file(), "{} is not a regular file", path.display());
    }
}

#[test]
fn guard_install_leaves_no_temp_entries_after_clean_install() {
    // Joins Land guard_install/write_replacing -> Gitio common_dir/hooks_path -> real directory contents.
    let (_repo, root) = repo();

    guard_install(&git(&root), &refs()).unwrap();

    assert_eq!(fs::read_to_string(root.join(".git/mortar/guard.json")).unwrap(), CONFIG);
    for directory in [root.join(".git/hooks"), root.join(".git/mortar")] {
        for entry in fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            assert!(!path.file_name().unwrap().to_string_lossy().contains(".mortar-tmp"), "temp entry remains: {}", path.display());
        }
    }
}

#[test]
fn guard_install_does_not_reuse_crash_left_fixed_temp() {
    // Joins Land guard_install/write_replacing -> Gitio common_dir/hooks_path -> stale-file preservation.
    let (_repo, root) = repo();
    let hooks = root.join(".git/hooks");
    fs::create_dir_all(&hooks).unwrap();
    let stale = hooks.join("pre-push.mortar-tmp");
    fs::write(&stale, b"stale\n").unwrap();
    fs::set_permissions(&stale, fs::Permissions::from_mode(0o644)).unwrap();

    guard_install(&git(&root), &refs()).unwrap();

    assert_eq!(fs::read_to_string(hooks.join("pre-push")).unwrap(), PRE_PUSH);
    assert_eq!(hook_mode(&hooks.join("pre-push")), 0o755);
    assert!(fs::symlink_metadata(hooks.join("pre-push")).unwrap().file_type().is_file());
    assert_eq!(fs::read(&stale).unwrap(), b"stale\n");
    assert_eq!(hook_mode(&stale), 0o644);
}
