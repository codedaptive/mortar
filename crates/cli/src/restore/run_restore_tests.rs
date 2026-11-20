//! run_restore: no op id, an op id with no ref, refs of two zones, a zone's ref with a landing ref and a ref that is
//! neither a zone's nor a landing ref are each restore's usage (exit 4, next restore's usage line), returned before any
//! lock or git run.
//! Unit tests: restore_lock and usage_report are plain functions, used directly; the context is built by hand with a cwd
//! that does not exist, so a run that took a lock or ran git would answer a refusal or a failure, not exit 4. The
//! BadMove case, with real git, is an integration test in crates/cli/tests/log_restore_runs/run_restore_tests.rs.

use std::collections::BTreeMap;
use std::path::Path;

use super::run_restore;
use crate::args::{find_verb, ParsedArgs};
use crate::config::Config;
use crate::report::{ExitCode, Report};
use crate::shell::ShellContext;

const RESTORE_USAGE: &str =
    "mr restore <op id> <full ref name>... --actor <name> [--key <operation key>] [--json]";

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
    let report = run_restore(&context(rest), Path::new("/nonexistent/mortar-run-restore-zut"));
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
