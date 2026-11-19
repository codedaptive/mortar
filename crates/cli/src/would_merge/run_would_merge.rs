//! run_would_merge: `mr would-merge <commit or change ID>`, whether a squash would merge cleanly into the
//! integration tip, under the land lock. No ref moves and no commit is written (git's trial merge
//! writes tree objects nothing keeps); --key names the lock holder only, never passed to the op log.
//! No sibling test: it takes a real lock and runs git and land; parse_would_merge_args_tests,
//! would_merge_report_tests and landing's pick_squash_ref_tests, squash_pick_report_tests and
//! squash_error_report_tests cover its decisions.
//! Validation tests to write (not written here): would-merge yes and no, by change ID
//! and by commit; a change ID no squash ref carries, exit 1; two squash refs carrying one change ID, exit 1; a
//! squash ref holding a commit read_squash refuses; the land lock held by a live pid, exit 3; without a config,
//! exit 1, next mr init.

use std::path::Path;

use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{parse_would_merge_args, would_merge_report};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::landing::{resolve_squash, squash_error_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "would-merge";

/// The squash named by parse_would_merge_args (an Err is would-merge's usage report, exit 4, before the verb runs
/// git); then, under the land lock after recover (locked_act), keyed by --key or op_key("would-merge", the common
/// dir), the squash read by resolve_squash and land's would_merge of it into the config's integration branch,
/// reported by would_merge_report or, when refused, squash_error_report.
pub fn run_would_merge(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before would-merge runs");
    };
    let name = match parse_would_merge_args(&args.rest) {
        Ok(name) => name,
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
        let squash = resolve_squash(&git, VERB, &name)?;
        Ok(match land::would_merge(&git, &guard_refs_for(config), &squash) {
            Ok(answer) => would_merge_report(&squash, &answer),
            Err(error) => squash_error_report(VERB, &error),
        })
    })
    .unwrap_or_else(|report| report)
}
