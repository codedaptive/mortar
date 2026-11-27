//! run_commit_files: `mr commit-files <path>... [--message <text>]`, named paths of the folder it runs in committed
//! onto the branch that folder stands for: integration in the integration folder (run_commit_files_integration),
//! zone/<zone> in a zone folder (run_commit_files_zone). The folder's other changes are left as they are.
//! No sibling test: it runs git and reads the folder record; parse_commit_files_args_tests,
//! default_commit_message_tests and commit_files_folder_refusal_tests cover its decisions. The integration tests
//! crates/cli/tests/folder_runs/run_commit_files_tests.rs (a zone folder committed, repeated as same and refused off
//! its tip; a request folder refused) and crates/cli/tests/land_runs/run_commit_files_tests.rs (the integration folder
//! after a landing, published with it; without a landing, refused) cover it through the binary.

use std::path::Path;

use gitio::Git;

use super::run_commit_files_integration::run_commit_files_integration;
use super::run_commit_files_zone::run_commit_files_zone;
use super::{commit_files_folder_refusal, default_commit_message, parse_commit_files_args};
use crate::args::{usage_report, UsageError};
use crate::context::{context_report, read_folder_context, repository_error, repository_report};
use crate::integration_folder::integration_folder;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "commit-files";

/// The paths and --message text read by parse_commit_files_args (an Err is commit-files' usage report, exit 2,
/// before any git); the message is the --message text, else default_commit_message. Then the top of `cwd`'s
/// worktree: when it is the integration folder (integration_folder, both paths canonicalized), the integration run;
/// else the folder record (read_folder_context; none is context_report), a request or attempt folder refused by
/// commit_files_folder_refusal, and a zone folder's run.
pub fn run_commit_files(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let (paths, message) = match parse_commit_files_args(&args.rest) {
        Ok(parsed) => parsed,
        Err(reason) => return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json }),
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before commit-files runs");
    };
    let message = message.unwrap_or_else(|| default_commit_message(&paths));
    let git = Git::new(cwd);
    let top = match git.toplevel() {
        Ok(top) => top,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    match integration_folder(&git, &config.integration_branch) {
        Ok(Some(folder)) if same_path(&folder, &top) => {
            return run_commit_files_integration(context, config, &top, &paths, &message);
        }
        Ok(_) => {}
        Err(reason) => return failure_report(VERB, &reason),
    }
    let folder = match read_folder_context(cwd) {
        Ok(folder) => folder,
        Err(error) => return context_report(VERB, &error),
    };
    if let Some(refusal) = commit_files_folder_refusal(&folder.toplevel, &folder.record.kind) {
        return refusal;
    }
    run_commit_files_zone(context, &folder, &paths, &message)
}

/// Whether both paths canonicalize to the same path; git's worktree list and rev-parse may name one folder through
/// different links (/var and /private/var on macOS). A path that cannot be canonicalized is the same as none.
fn same_path(one: &Path, other: &Path) -> bool {
    one.canonicalize().ok().zip(other.canonicalize().ok()).is_some_and(|(one, other)| one == other)
}
