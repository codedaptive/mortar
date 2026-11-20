//! run_uninstall: `mr uninstall`: reads its arguments and HOME, then uninstall_in.
//! No sibling test: it reads the process's environment. Its body past HOME is uninstall_in, which the install integration tests in
//! install/integration/uninstall.rs and uninstall_partial.rs cover; the command line itself is an integration test
//! through the binary to write (not written here).

use std::path::Path;

use crate::args::{no_arguments_report, usage_report, UsageError};
use crate::report::Report;
use crate::shell::ShellContext;

use super::{home_unset_refusal, uninstall_in};

/// Uninstall's report, or a usage report (`--key` given, since uninstall records no op, or any argument) or a
/// refusal (HOME unset or empty, owned by the person uninstalling Mortar) before uninstall_in.
pub fn run_uninstall(context: &ShellContext) -> Report {
    let args = &context.args;
    if args.key.is_some() {
        return usage_report(&UsageError {
            verb: Some("uninstall".to_string()),
            reason: "uninstall takes no --key".to_string(),
            json: args.json,
        });
    }
    if let Some(report) = no_arguments_report("uninstall", &args.rest, args.json) {
        return report;
    }
    match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        Some(home) => uninstall_in(Path::new(&home)),
        None => home_unset_refusal("uninstall", "the person uninstalling Mortar"),
    }
}
