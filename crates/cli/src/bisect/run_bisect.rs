//! run_bisect: `mr bisect <last pass> <failed> --timeout <seconds> -- <command>...`, the first failing landing
//! on origin's first-parent line named under the land lock. land's bisect probes each commit in a
//! throwaway detached worktree and leaves no bisect state; --key names the lock holder only, never passed to the
//! op log.
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_bisect_args_tests,
//! bisect_report_tests, bisect_error_report_tests, probe_line_tests and culprit_line_tests cover its decisions.
//! Repo integration tests to write (cross-zone, not written here): a culprit found at a fast-forward
//! landing and at a merge landing, its change ID and zone named; a culprit landing without trailers; not found
//! because the failed commit passes, and because the last passing commit fails; a timed-out probe counted as a
//! fail and shown "timed out"; a last passing commit off the failed one's first-parent line refused naming both; a
//! failed commit off origin refused; a command that cannot start stops the search naming the commit; `--actor`
//! after `--` reaches the command as one of its words; no worktree or bisect state left after a run; the land lock
//! held by a live pid, exit 3; without a config, exit 1, next mr init.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{bisect_error_report, bisect_report, parse_bisect_args};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "bisect";

/// The words read by parse_bisect_args from the line's rest and its `--` mark (an Err is bisect's usage report,
/// exit 4, before any git). Then, under the land lock after recover (locked_act), keyed by --key or
/// op_key("bisect", the common dir), land's bisect over the config's origin branch, reported by bisect_report or,
/// when refused or failed, bisect_error_report.
pub fn run_bisect(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let words = match parse_bisect_args(&args.rest, args.dash_dash) {
        Ok(words) => words,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before bisect runs");
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let refs = guard_refs_for(config);
        Ok(match land::bisect(&git, &refs, &words.last_pass, &words.failed, &words.command, words.timeout) {
            Ok(report) => bisect_report(&report, &words.last_pass, &words.failed),
            Err(error) => bisect_error_report(&error),
        })
    })
    .unwrap_or_else(|report| report)
}
