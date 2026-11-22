//! run_squash: `mr squash <zone> [--message <text>]`, a configured zone's changes squashed onto origin as one commit
//! at refs/mortar/squash/<zone> under the land lock, its message led by the caller's text when one is given.
//! --key names the lock holder only, never passed to the op log: repeat safety comes from land's answers (Same).
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_squash_args_tests,
//! squash_report_tests and landing's squash_error_report_tests cover its decisions. The integration tests
//! crates/cli/tests/land_runs/run_squash_tests.rs and run_squash_message_tests.rs cover it through the binary.
//! Integration tests to write (not written here): the land lock held by a live pid,
//! exit 3; without a config, exit 1, next mr init; the SquashError Git, Oplog, ChangeId, Trailers, OpFailed and
//! OpRepeated texts when git or the op log fails for real; the acceptance checks of squash.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_squash_args, squash_report};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::landing::squash_error_report;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "squash";

/// The config's zone and the --message text read by parse_squash_args (an Err is squash's usage report, exit 4,
/// before the verb runs git); then, under the land lock after recover (locked_act), keyed by --key or
/// op_key("squash", the common dir), land's squash_with_message of that zone onto the config's origin branch as the
/// context's actor with that text (none without --message), reported by squash_report or, when refused,
/// squash_error_report.
pub fn run_squash(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before squash runs");
    };
    let (zone, message) = match parse_squash_args(&args.rest, config) {
        Ok(parsed) => parsed,
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
        let refs = guard_refs_for(config);
        Ok(match land::squash_with_message(&git, &refs, &zone, &context.actor, message.as_deref()) {
            Ok(answer) => squash_report(&zone, &answer),
            Err(error) => squash_error_report(VERB, &error),
        })
    })
    .unwrap_or_else(|report| report)
}
