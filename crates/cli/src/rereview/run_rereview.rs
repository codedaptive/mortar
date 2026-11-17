//! run_rereview: `mr rereview <reviewed commit> <new commit>`, the re-review diff of two squashes of one change
//! (SPEC-cli 06): its words read, then land's rereview under the land lock after recover, as every landing verb
//! runs (S-16), though rereview moves no ref and records no op. --key names the lock holder only,
//! never passed to the op log (S-132).
//! No sibling test (S-104): it takes a real lock and runs git and land; parse_rereview_args_tests,
//! rereview_report_tests and rereview_error_report_tests cover its decisions. The RIT
//! crates/cli/tests/cli_rit/run_rereview_tests.rs covers it through the binary: two squashes of one change are "rereviewed"
//! with both ids and only what changed; squashes of two changes are refused exit 1 "not-same-change"; one word,
//! three words and a word that is not a full commit id are exit 4 with rereview's usage line as next.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119): the land lock held by a live pid, exit
//! 3 (oplog's locks); without a config, exit 1, next mr init (the shell's load_config); RereviewError Git,
//! Trailers and NoParent (a root commit) through the binary; ACCEPT-1 check 9.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_rereview_args, rereview_error_report, rereview_report};
use crate::args::{usage_report, UsageError};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "rereview";

/// The words after the verb read by parse_rereview_args (an Err is rereview's usage report, exit 4, before any
/// git); then, under the land lock keyed by --key or op_key("rereview", the common dir), land's rereview of the
/// two commits: its diff is rereview_report, its refusal or failure rereview_error_report.
pub fn run_rereview(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let (reviewed, new) = match parse_rereview_args(&args.rest) {
        Ok(commits) => commits,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    if context.config.is_none() {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before rereview runs");
    }
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        land::rereview(&git, &reviewed, &new)
            .map(|diff| rereview_report(&reviewed, &new, &diff))
            .map_err(|error| rereview_error_report(&error))
    })
    .unwrap_or_else(|report| report)
}
