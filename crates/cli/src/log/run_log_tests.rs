//! run_log: a filter word log does not know, a second --ref and a --ref without a value are each log's usage (exit
//! 2, next log's usage line), returned before the op log or git is read.
//! Unit tests: log_filters and usage_report are plain functions, used directly; the context is built by hand. cwd does not
//! exist, so a run that read git would answer a refusal or a failure, not exit 2. The listing cases, with a real op
//! log, are integration tests in crates/cli/tests/log_restore_runs/run_log_tests.rs.

use std::path::Path;

use super::run_log;
use crate::args::{find_verb, ParsedArgs};
use crate::report::ExitCode;
use crate::shell::ShellContext;

const LOG_USAGE: &str = "mr log [--ref <full ref name>] [--actor <name>] [--key <operation key>] [--json]";

fn context(rest: &[&str]) -> ShellContext {
    ShellContext {
        args: ParsedArgs {
            verb: find_verb("log").expect("log is a verb"),
            rest: rest.iter().map(ToString::to_string).collect(),
            dash_dash: None,
            json: false,
            key: None,
            actor: None,
        },
        actor: String::new(),
        folder: None,
        config: None,
        protected: Vec::new(),
    }
}

fn assert_usage(rest: &[&str], reason: &str) {
    let report = run_log(&context(rest), Path::new("/nonexistent/mortar-run-log-zut"));
    assert_eq!(report.verb, "log");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.outcome, "usage");
    assert_eq!(report.lines[0], reason);
    assert_eq!(report.next, LOG_USAGE);
    assert!(report.fields.is_empty());
}

#[test]
fn a_word_that_is_not_a_filter_is_logs_usage() {
    assert_usage(&["oops"], "unknown word \"oops\"");
}

#[test]
fn a_second_ref_is_logs_usage() {
    assert_usage(&["--ref", "refs/heads/a", "--ref=refs/heads/b"], "--ref is given twice");
}

#[test]
fn a_ref_without_a_value_is_logs_usage() {
    assert_usage(&["--ref"], "--ref needs a value");
}
