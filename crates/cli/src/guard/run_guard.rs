//! run_guard: `mr guard (install | status)`, the harness verb that installs Mortar's git hooks and guard.json and
//! says whether they are in place: its words read, then the sub-verb's runner.
//! No sibling test: it reaches git, the op log and land through its runners; parse_guard_args_tests covers
//! its words. The validation tests crates/cli/tests/validation/run_guard_install_tests.rs and run_guard_status_tests.rs cover it
//! through the binary: no sub-verb and an unknown sub-verb exit 4 with guard's usage line as next, nothing written.
//! Validation tests to write (not written here): without a config, exit 1, next
//! mr init (the shell's load_config).

use std::path::Path;

use super::{parse_guard_args, run_guard_install, run_guard_status, GuardCommand};
use crate::args::{usage_report, UsageError};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "guard";

/// The words after the verb read by parse_guard_args (an Err is guard's usage report, exit 4, before the verb runs
/// git); then install's or status's runner with the config the shell loaded.
pub fn run_guard(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let command = match parse_guard_args(&args.rest) {
        Ok(command) => command,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before guard runs");
    };
    match command {
        GuardCommand::Install => run_guard_install(context, config, cwd),
        GuardCommand::Status => run_guard_status(context, config, cwd),
    }
}
