//! run_restore: a full ref name git refuses (refs/heads/req/core/a..b) is oplog's BadMove, exit 1, before any ref is
//! read: no ref is written and the op log gains no record.
//! Validation test: `mr --json --actor harness-1 restore` runs as a child of the mortar binary in a throwaway
//! repository (testkit's TestRepo) with one commit, one done op and the config `mr init` writes first; the setup git
//! and the children run with GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1. The cli lane owns it.
//! restore's usage cases are ZUTs in crates/cli/src/restore/run_restore_tests.rs.

use std::path::Path;
use std::process::{Command, Output, Stdio};

use gitio::{Git, RunOpts};
use oplog::{Op, RefMove};
use testkit::TestRepo;

use crate::json::check;

fn mortar(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(cwd).env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", cwd).env("TMPDIR", cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(args).stdin(Stdio::null()).output().unwrap()
}

#[test]
fn a_ref_name_git_refuses_is_oplogs_bad_move_with_nothing_written() {
    let repo = TestRepo::new().unwrap();
    let mut git = Git::new(&repo.path);
    git.env = vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ("GIT_AUTHOR_NAME".into(), "Test Author".into()),
        ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
        ("GIT_COMMITTER_NAME".into(), "Test Committer".into()),
        ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
    ];
    std::fs::write(repo.path.join("file"), b"base\n").unwrap();
    git.run(&["add", "-A"], &RunOpts::default()).unwrap();
    git.run(&["commit", "-qm", "base"], &RunOpts::default()).unwrap();
    let base = git.rev_parse("HEAD").unwrap().unwrap();
    let common = git.common_dir().unwrap();
    let init = mortar(&repo.path, &[
        "--actor", "harness", "init", "--origin", "main", "--integration", "integration", "--remote", "origin",
        "--floor-bytes", "0",
    ]);
    assert!(init.status.success(), "init: {init:?}");
    oplog::recover(&git, &common).unwrap();
    let op = Op {
        op_id: oplog::new_op_id(),
        key: "fixture-task".to_string(),
        kind: "fixture".to_string(),
        actor: "fixture".to_string(),
        reason: "a done op to restore to".to_string(),
        at: 1,
        moves: vec![RefMove { refname: "refs/heads/req/core/task".to_string(), old: None, new: base }],
    };
    oplog::apply(&git, &common, &op).unwrap();
    let before = oplog::read_op_log(&common).unwrap().records.len();

    let output =
        mortar(&repo.path, &["--json", "--actor", "harness-1", "restore", &op.op_id, "refs/heads/req/core/a..b"]);

    let report = check(output, "restore", 1, "refused");
    assert_eq!(
        report.strings("lines")[0],
        "op cannot be applied: ref \"refs/heads/req/core/a..b\" is not a valid ref name (git check-ref-format \
         refused it); fix the op's moves and run again"
    );
    assert_eq!(report.text("next"), "name \"refs/heads/req/core/a..b\" by its full, valid ref name, then repeat");
    assert!(!report.text("next").contains("mr log --ref"), "{}", report.text("next"));
    assert_eq!(git.read_ref("refs/heads/req/core/a..b").unwrap(), None);
    assert_eq!(oplog::read_op_log(&common).unwrap().records.len(), before);
}
