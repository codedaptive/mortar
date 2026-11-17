//! run_guard: `mortar guard (install | status)`, the harness verb that installs Mortar's git hooks and guard.json and
//! says whether they are in place (SPEC-cli 06): its words read, then the sub-verb's runner.
//! No sibling test (S-104): it reaches git, the op log and land through its runners; parse_guard_args_tests covers
//! its words. The RITs crates/cli/tests/cli_rit/run_guard_install_tests.rs and run_guard_status_tests.rs cover it
//! through the binary: no sub-verb and an unknown sub-verb exit 4 with guard's usage line as next, nothing written.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119): without a config, exit 1, next
//! mortar init (the shell's load_config).

use std::path::Path;

use super::{parse_guard_args, run_guard_install, run_guard_status, GuardCommand};
use crate::args::{usage_report, UsageError};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "guard";

/// The words after the verb read by parse_guard_args (an Err is guard's usage report, exit 4, before any git); then
/// install's or status's runner with the config the shell loaded.
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
