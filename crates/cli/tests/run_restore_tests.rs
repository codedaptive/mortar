//! run_restore: no op id, an op id with no ref, refs of two zones, a zone's ref with a landing ref and a ref that is
//! neither a zone's nor a landing ref are each restore's usage (exit 4, next restore's usage line), returned before any
//! lock or git run (S-113(1), S-115(2)). A full ref name git refuses (refs/heads/req/core/a..b) is oplog's BadMove,
//! exit 1, before any ref is read: no ref is written and the op log gains no record.
//! ZIT (S-79 as replaced: the line is the file): it calls restore_lock, usage_report, restore_moves and
//! restore_error_report for real. The usage tests build the context by hand with a cwd that does not exist, so a run
//! that took a lock or ran git would answer a refusal or a failure, not exit 4. The BadMove test runs real git in a
//! throwaway repository with one commit and one done op; its setup git sets GIT_CONFIG_GLOBAL=/dev/null and
//! GIT_CONFIG_NOSYSTEM=1; run_restore's own git runs with the process environment.

use std::collections::BTreeMap;
use std::path::Path;

use cli::{find_verb, run_restore, Config, ExitCode, ParsedArgs, Report, ShellContext};
use gitio::{Git, RunOpts};
use oplog::{Op, RefMove};
use testkit::TestRepo;

const RESTORE_USAGE: &str =
    "mortar restore <op id> <full ref name>... --actor <name> [--key <operation key>] [--json]";

fn context(rest: &[&str]) -> ShellContext {
    let config = Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 5_242_880,
        floor_bytes: 0,
        formatter: None,
    };
    ShellContext {
        args: ParsedArgs {
            verb: find_verb("restore").expect("restore is a verb"),
            rest: rest.iter().map(ToString::to_string).collect(),
            dash_dash: None,
            json: false,
            key: None,
            actor: Some("harness-1".to_string()),
        },
        actor: "harness-1".to_string(),
        folder: None,
        config: Some(config),
        protected: Vec::new(),
    }
}

fn usage(rest: &[&str]) -> Report {
    let report = run_restore(&context(rest), Path::new("/nonexistent/mortar-run-restore-zit"));
    assert_eq!(report.verb, "restore");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.outcome, "usage");
    assert_eq!(report.next, RESTORE_USAGE);
    assert_eq!(report.owner, None);
    assert!(report.fields.is_empty());
    report
}

#[test]
fn no_op_id_is_restores_usage() {
    assert_eq!(usage(&[]).lines[0], "restore needs an op id, then at least one full ref name");
}

#[test]
fn an_op_id_with_no_ref_is_restores_usage() {
    assert_eq!(usage(&["op-1"]).lines[0], "restore needs at least one full ref name after the op id");
}

#[test]
fn refs_of_two_zones_are_restores_usage_naming_both() {
    let line = &usage(&["op-1", "refs/heads/zone/core", "refs/heads/req/cli/fix-1"]).lines[0];
    assert!(line.contains("refs/heads/zone/core"), "{line}");
    assert!(line.contains("refs/heads/req/cli/fix-1"), "{line}");
}

#[test]
fn a_zone_ref_with_a_landing_ref_is_restores_usage_naming_both() {
    let line = &usage(&["op-1", "refs/heads/integration", "refs/mortar/snap/core/fix-1"]).lines[0];
    assert!(line.contains("refs/heads/integration"), "{line}");
    assert!(line.contains("refs/mortar/snap/core/fix-1"), "{line}");
}

#[test]
fn a_ref_of_no_zone_and_not_landing_is_restores_usage_naming_it() {
    let line = &usage(&["op-1", "refs/heads/feature/x"]).lines[0];
    assert!(line.contains("refs/heads/feature/x"), "{line}");
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

    let report = run_restore(&context(&[&op.op_id, "refs/heads/req/core/a..b"]), &repo.path);

    assert_eq!(report.exit, ExitCode::Refused, "{report:?}");
    assert_eq!(report.outcome, "refused");
    assert_eq!(
        report.lines[0],
        "op cannot be applied: ref \"refs/heads/req/core/a..b\" is not a valid ref name (git check-ref-format \
         refused it); fix the op's moves and run again"
    );
    assert_eq!(report.next, "name \"refs/heads/req/core/a..b\" by its full, valid ref name, then repeat");
    assert!(!report.next.contains("mortar log --ref"), "{}", report.next);
    assert_eq!(git.read_ref("refs/heads/req/core/a..b").unwrap(), None);
    assert_eq!(oplog::read_op_log(&common).unwrap().records.len(), before);
}
