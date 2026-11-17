//! save_run_for: a request folder gives its request branch, attempt folder k its try branch and target attempt k
//! (no refusal: a save runs in either), both with the zone's join lock and save's op key (--key wins); a missing
//! folder or config, or a record naming no request, is save's failure. Git::new runs nothing, so no git runs here.

use std::collections::BTreeMap;
use std::path::PathBuf;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;
use oplog::LockName;
use snapshot::SnapshotTarget;

use super::save_run_for;
use crate::args::{find_verb, ParsedArgs};
use crate::config::Config;
use crate::context::FolderContext;
use crate::report::{ExitCode, Report};
use crate::shell::ShellContext;

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: Some("zone/core".to_string()),
    }
}

fn config() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 5_242_880,
        floor_bytes: 0,
        formatter: None,
    }
}

fn context(toplevel: &str, record: FolderRecord, key: Option<&str>) -> ShellContext {
    ShellContext {
        args: ParsedArgs {
            verb: find_verb("save").expect("save is a verb"),
            rest: vec!["fix the parser".to_string()],
            dash_dash: None,
            json: false,
            key: key.map(str::to_string),
            actor: None,
        },
        actor: "core/fix-1".to_string(),
        folder: Some(FolderContext {
            toplevel: PathBuf::from(toplevel),
            git_dir: PathBuf::from("/r/.git/worktrees/fix-1"),
            common: PathBuf::from("/r/.git"),
            record,
        }),
        config: Some(config()),
        protected: Vec::new(),
    }
}

fn request_context(key: Option<&str>) -> ShellContext {
    context("/w/fix-1", record(FolderKind::Request, Some("fix-1")), key)
}

fn target(attempt: Option<u32>) -> SnapshotTarget {
    SnapshotTarget {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        attempt,
        actor: "core/fix-1".to_string(),
    }
}

/// failure_report's shape for save, whatever words name the missing part.
fn assert_save_failure(report: &Report) {
    assert_eq!(report.verb, "save");
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(report.outcome, "failed");
    assert_eq!(report.lines[1], "nothing half-done is left");
    assert_eq!(report.next, "repeat the same command");
}

#[test]
fn a_request_folder_gives_its_request_branch_lock_target_and_key() {
    let context = request_context(None);
    let run = save_run_for(&context, "save").expect("a request folder saves");
    assert_eq!(run.request_ref, "refs/heads/req/core/fix-1");
    assert_eq!(run.target, target(None));
    assert_eq!(run.lock, LockName::Join("core".to_string()));
    assert_eq!(run.key, "save-42ddddff775b3a20");
    assert_eq!(run.git.repo, PathBuf::from("/w/fix-1"));
    assert_eq!(run.settings.cap_bytes, 5_242_880);
}

#[test]
fn the_key_flag_is_the_key() {
    let context = request_context(Some("harness-7"));
    assert_eq!(save_run_for(&context, "save").expect("a request folder saves").key, "harness-7");
}

#[test]
fn attempt_folder_2_gives_its_try_branch_and_attempt_2_and_is_not_refused() {
    let context = context("/w/fix-1/try-2", record(FolderKind::Attempt(2), Some("fix-1")), None);
    let run = save_run_for(&context, "save").expect("an attempt folder saves");
    assert_eq!(run.request_ref, "refs/heads/try/core/fix-1/2");
    assert_eq!(run.target, target(Some(2)));
    assert_eq!(run.lock, LockName::Join("core".to_string()));
    assert_eq!(run.key, "save-cc5873e9a96bc59b");
}

#[test]
fn no_folder_is_a_failure() {
    let mut context = request_context(None);
    context.folder = None;
    assert_save_failure(&save_run_for(&context, "save").unwrap_err());
}

#[test]
fn no_config_is_a_failure() {
    let mut context = request_context(None);
    context.config = None;
    assert_save_failure(&save_run_for(&context, "save").unwrap_err());
}

#[test]
fn a_record_naming_no_request_is_a_failure() {
    let context = context("/w/core", record(FolderKind::Zone, None), None);
    assert_save_failure(&save_run_for(&context, "save").unwrap_err());
}

/// undo builds its run here too: the same branch, target and lock, its own key and its own failures' verb.
#[test]
fn undo_gets_the_same_run_with_its_own_key_and_verb() {
    let context = request_context(None);
    let run = save_run_for(&context, "undo").expect("a request folder undoes");
    assert_eq!(run.request_ref, "refs/heads/req/core/fix-1");
    assert_eq!(run.target, target(None));
    assert_eq!(run.lock, LockName::Join("core".to_string()));
    assert!(run.key.starts_with("undo-"), "{}", run.key);
    assert_ne!(run.key, "save-42ddddff775b3a20");
    let none = context_without_folder_for_undo();
    assert_eq!(save_run_for(&none, "undo").unwrap_err().verb, "undo");
}

fn context_without_folder_for_undo() -> ShellContext {
    ShellContext { folder: None, ..request_context(None) }
}
