//! run_commit_files_integration: `mr commit-files` in the integration folder, named paths committed onto integration
//! inside the landing in flight by land's commit_files, under the land lock.
//! --key names the lock holder only, never passed to the op log: land's commit_files draws its own op key, and
//! repeat safety comes from its answers (Same).
//! No sibling test: it takes a real lock and runs git, the op log and land; commit_files_report_tests and
//! commit_files_error_report_tests cover its decisions. The integration test
//! crates/cli/tests/land_runs/run_commit_files_tests.rs covers it through the binary: a commit after a real landing,
//! parented by the landed commit and pushed by publish; without a landing, refused with integration unmoved.

use std::path::{Path, PathBuf};

use gitio::Git;
use land::CommitFilesAnswer;
use oplog::{KillPidProbe, LockName};

use super::{commit_files_error_report, commit_files_report};
use crate::config::Config;
use crate::guard::guard_refs_for;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "commit-files";
const NEXT: &str = "mr publish --actor <name> pushes it with the landing; mr back-out --actor <name> drops both";

/// `top` is the integration folder's top. Under the land lock after recover (locked_act), keyed by --key or
/// op_key("commit-files", the common dir): refs/heads/<integration>'s tip read (git failing or the branch missing is
/// failure_report), then land's commit_files of `paths` with `message` as the context's actor. Committed and Same
/// are commit_files_report from that tip; a refusal is commit_files_error_report.
pub(super) fn run_commit_files_integration(
    context: &ShellContext,
    config: &Config,
    top: &Path,
    paths: &[PathBuf],
    message: &str,
) -> Report {
    let git = Git::new(top);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let integration = &config.integration_branch;
    let integration_ref = format!("refs/heads/{integration}");
    let key = op_key(context.args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let old = match git.read_ref(&integration_ref) {
            Ok(Some(tip)) => tip,
            Ok(None) => return Err(failure_report(VERB, &format!("{integration_ref} does not exist"))),
            Err(_) => return Err(failure_report(VERB, &format!("git could not read {integration_ref}"))),
        };
        Ok(match land::commit_files(&git, &guard_refs_for(config), paths, message, &context.actor) {
            Ok(CommitFilesAnswer::Committed { commit }) => {
                commit_files_report(&integration_ref, &old, Some(&commit), paths, NEXT.to_string())
            }
            Ok(CommitFilesAnswer::Same) => commit_files_report(&integration_ref, &old, None, paths, NEXT.to_string()),
            Err(error) => commit_files_error_report(&error, integration),
        })
    })
    .unwrap_or_else(|report| report)
}
