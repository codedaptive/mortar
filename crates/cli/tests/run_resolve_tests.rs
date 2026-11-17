//! run_resolve: an attempt folder is the S-91 refusal with nothing run, and an argument is resolve's usage; both
//! return before any lock or git run.
//! ZIT (S-79 as replaced: the line is the file): it calls request_run_for, attempt_refusal and usage_report for real.
//! The context is built by hand; Git::new runs nothing.

use std::collections::BTreeMap;
use std::path::PathBuf;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use cli::{find_verb, run_resolve, Config, ExitCode, FolderContext, ParsedArgs, Report, ShellContext};

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
            verb: find_verb("resolve").expect("resolve is a verb"),
            rest,
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

#[test]
fn an_attempt_folder_is_refused_with_nothing_run() {
    assert_eq!(
        run_resolve(&context(FolderKind::Attempt(2), Vec::new())),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "this is attempt folder 2 of request fix-1: mortar resolve runs in the request's own folder"
                    .to_string(),
                "nothing ran".to_string(),
            ],
            owner: Some("this request".to_string()),
            next: "keep or drop the attempts, then mortar join in the request's folder".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn an_argument_is_resolves_usage() {
    assert_eq!(
        run_resolve(&context(FolderKind::Request, vec!["now".to_string()])),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Usage,
            outcome: "usage".to_string(),
            lines: vec!["resolve takes no arguments".to_string(), "verbs: back-out, folder, guard, guard-hook, init, join, land, log, need, publish, rereview, resolve, restore, save, squash, status, would-merge".to_string()],
            owner: None,
            next: "mortar resolve [--key <operation key>] [--json]".to_string(),
            fields: Vec::new(),
        }
    );
}
