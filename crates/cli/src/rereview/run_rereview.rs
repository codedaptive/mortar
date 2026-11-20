//! run_rereview: `mr rereview <reviewed commit> <new commit>`, the re-review diff of two squashes of one change:
//! its words read, then land's rereview under the land lock after recover, as every landing verb
//! runs, though rereview moves no ref and records no op. --key names the lock holder only,
//! never passed to the op log.
//! No sibling test: it takes a real lock and runs git and land; parse_rereview_args_tests,
//! rereview_report_tests and rereview_error_report_tests cover its decisions. The integration test
//! crates/cli/tests/land_runs/run_rereview_tests.rs covers it through the binary: two squashes of one change are "rereviewed"
//! with both ids and only what changed; squashes of two changes are refused exit 1 "not-same-change"; one word,
//! three words and a word that is not a full commit id are exit 4 with rereview's usage line as next.
//! crates/cli/tests/land_runs/run_rereview_savings_tests.rs covers its savings: the baseline equal to git diff of
//! the two commits, in the plain answer, --json and savings.jsonl.
//! Integration tests to write (not written here): the land lock held by a live pid, exit
//! 3 (oplog's locks); without a config, exit 1, next mr init (the shell's load_config); RereviewError Git,
//! Trailers and NoParent (a root commit) through the binary; the acceptance check of rereview.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_rereview_args, rereview_error_report, rereview_queries, rereview_report};
use crate::args::{usage_report, UsageError};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::savings::with_counted_savings;
use crate::shell::ShellContext;

const VERB: &str = "rereview";

/// The words after the verb read by parse_rereview_args (an Err is rereview's usage report, exit 4, before any
/// git); then, under the land lock keyed by --key or op_key("rereview", the common dir), land's rereview of the
/// two commits: its diff is rereview_report, its refusal or failure rereview_error_report. A Done report gets its
/// savings by with_counted_savings, outside the lock since it only reads: one more git read, git diff of the two
/// commits (rereview_queries), its output characters counted by Git::output_chars. A count that fails drops the
/// savings, so the answer goes out without the line; a lock refusal, a usage report or a failure runs no count.
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
    let report = locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        land::rereview(&git, &reviewed, &new)
            .map(|diff| rereview_report(&reviewed, &new, &diff))
            .map_err(|error| rereview_error_report(&error))
    })
    .unwrap_or_else(|report| report);
    with_counted_savings(report, &rereview_queries(&reviewed, &new), &|query| git.output_chars(cwd, query))
}
