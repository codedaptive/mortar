//! request_run_for: an attempt folder is the S-91 refusal for join and for resolve; a request folder gives its ref,
//! its zone's join lock, the snapshot target and the derived op key; an argument is the verb's usage.
//! ZIT (S-79 as replaced: the line is the file): it calls join/attempt_refusal and args/usage_report for real.

use std::collections::BTreeMap;
use std::path::PathBuf;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;
use oplog::LockName;
use snapshot::SnapshotTarget;

use cli::{find_verb, request_run_for, Config, ExitCode, FolderContext, ParsedArgs, Report, ShellContext};

fn context(kind: FolderKind, rest: Vec<String>) -> ShellContext {
    let record = FolderRecord {
        kind,
        zone: "core".to_string(),
        request: Some("fix-1".to_string()),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: Some("zone/core".to_string()),
    };
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
            verb: find_verb("join").expect("join is a verb"),
            rest,
            dash_dash: None,
            json: false,
            key: None,
            actor: None,
        },
        actor: "core/fix-1".to_string(),
        folder: Some(FolderContext {
            toplevel: PathBuf::from("/w/fix-1"),
            git_dir: PathBuf::from("/r/.git/worktrees/fix-1"),
            common: PathBuf::from("/r/.git"),
            record,
        }),
        config: Some(config),
        protected: Vec::new(),
    }
}

fn attempt_refused(verb: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("this is attempt folder 3 of request fix-1: mr {verb} runs in the request's own folder"),
            "nothing ran".to_string(),
        ],
        owner: Some("this request".to_string()),
        next: "keep or drop the attempts, then mr join in the request's folder".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn join_in_an_attempt_folder_is_refused() {
    let context = context(FolderKind::Attempt(3), Vec::new());
    assert_eq!(request_run_for(&context, "join").unwrap_err(), attempt_refused("join"));
}

#[test]
fn resolve_in_an_attempt_folder_is_refused() {
    let context = context(FolderKind::Attempt(3), Vec::new());
    assert_eq!(request_run_for(&context, "resolve").unwrap_err(), attempt_refused("resolve"));
}

#[test]
fn a_request_folder_gives_its_ref_lock_target_and_key() {
    let context = context(FolderKind::Request, Vec::new());
    let run = request_run_for(&context, "join").expect("a request folder runs");
    assert_eq!(run.request_ref, "refs/heads/req/core/fix-1");
    assert_eq!(run.lock, LockName::Join("core".to_string()));
    assert_eq!(run.key, "join-ae50f85dfaa45ab9");
    assert_eq!(
        run.target,
        SnapshotTarget {
            zone: "core".to_string(),
            request: "fix-1".to_string(),
            attempt: None,
            actor: "core/fix-1".to_string(),
        }
    );
    assert_eq!(run.git.repo, PathBuf::from("/w/fix-1"));
}

#[test]
fn an_argument_is_the_verbs_usage() {
    let context = context(FolderKind::Request, vec!["now".to_string()]);
    let report = request_run_for(&context, "join").unwrap_err();
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.verb, "join");
    assert_eq!(report.lines[0], "join takes no arguments");
    assert_eq!(report.next, "mr join [--key <operation key>] [--json]");
}
