//! run_withdraw: `mr withdraw <zone>`, a configured zone's squash withdrawn before it lands, its squash ref deleted
//! under the land lock. --key names the lock holder only, never passed to the op log: land's withdraw draws its own
//! op key per attempt, and a repeat answers Absent.
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_withdraw_args_tests,
//! withdraw_report_tests and landing's withdraw_error_report_tests cover its decisions. The integration test
//! crates/cli/tests/land_runs/run_withdraw_tests.rs covers it through the binary.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_withdraw_args, withdraw_report};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::landing::withdraw_error_report;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "withdraw";

/// The config's zone read by parse_withdraw_args (an Err is withdraw's usage report, exit 4, before the verb runs
/// git); then, under the land lock after recover (locked_act), keyed by --key or op_key("withdraw", the common dir),
/// land's withdraw of that zone's squash, read against the config's origin branch, as the context's actor, reported
/// by withdraw_report or, when refused, withdraw_error_report.
pub fn run_withdraw(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before withdraw runs");
    };
    let zone = match parse_withdraw_args(&args.rest, config) {
        Ok(zone) => zone,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        Ok(match land::withdraw(&git, &guard_refs_for(config), &zone, &context.actor) {
            Ok(answer) => withdraw_report(&zone, &answer),
            Err(error) => withdraw_error_report(VERB, &error),
        })
    })
    .unwrap_or_else(|report| report)
}
