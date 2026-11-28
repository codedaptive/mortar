//! run_would_merge: `mr would-merge <commit or change ID>`, whether a squash would merge cleanly into the
//! integration tip, and `mr would-merge <squash> --with <squash>...`, whether it and the further squashes would
//! merge in order, each onto the result of the one before; both under the land lock. No ref moves (git's trial
//! merges write tree objects, and the --with form's merge commits, that nothing reaches); --key names the lock holder
//! only, never passed to the op log.
//! No sibling test: it takes a real lock and runs git and land; parse_would_merge_args_tests,
//! would_merge_report_tests, would_merge_with_report_tests and landing's pick_squash_ref_tests,
//! squash_pick_report_tests, squash_error_report_tests and land_error_report_tests cover its decisions.
//! The integration tests in crates/cli/tests/land_runs/land_flow/ cover it through the binary: would_merge.rs
//! would-merge yes and no, by change ID and by commit; a change ID no squash ref carries, exit 3; two squash refs
//! carrying one change ID, exit 3; a squash ref holding a commit read_squash refuses; its savings, the baseline equal
//! to git log -p of the squash over the integration branch, and none on a conflict; would_merge_with.rs ordered
//! squashes merging cleanly, a second squash conflicting with the first's result naming step 2, a change ID to
//! --with answered like its commit, a --with change ID no squash ref carries refused like the plain form, each with
//! the refs unchanged; landing.rs the land lock held by a live pid, exit 4, and without a config, exit 2, next mr
//! init.

use std::path::Path;

use gitio::Git;
use land::Squash;
use oplog::{KillPidProbe, LockName};

use super::{parse_would_merge_args, would_merge_queries, would_merge_report, would_merge_with_report};
use crate::args::{usage_report, UsageError};
use crate::config::Config;
use crate::guard::guard_refs_for;
use crate::landing::{land_error_report, resolve_squash, squash_error_report, SquashName};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::savings::with_counted_savings;
use crate::shell::ShellContext;

const VERB: &str = "would-merge";

/// The words read by parse_would_merge_args (an Err is would-merge's usage report, exit 2, before the verb runs
/// git); then, under the land lock after recover (locked_act), keyed by --key or op_key("would-merge", the common
/// dir), the squash read by resolve_squash. With no --with, land's would_merge of it into the config's integration
/// branch, reported by would_merge_report or, when refused, squash_error_report. A Done would_merge_report (would
/// merge) gets its savings by with_counted_savings, inside the lock where the squash is known: one more git read, git
/// log -p of the squash's commits not on refs/heads/<integration branch> (would_merge_queries), its output characters
/// counted by Git::output_chars. A count that fails drops the savings, so the answer goes out without the line; a
/// conflict answer (exit 1), a refusal or a failure runs no count. With --with, see with_answer.
pub fn run_would_merge(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before would-merge runs");
    };
    let words = match parse_would_merge_args(&args.rest) {
        Ok(words) => words,
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
        let squash = resolve_squash(&git, VERB, &words.squash)?;
        if !words.with.is_empty() {
            return with_answer(&git, config, &squash, &words.with);
        }
        Ok(match land::would_merge(&git, &guard_refs_for(config), &squash) {
            Ok(answer) => with_counted_savings(
                would_merge_report(&squash, &answer),
                &would_merge_queries(&config.integration_branch, &squash.commit),
                &|query| git.output_chars(cwd, query),
            ),
            Err(error) => squash_error_report(VERB, &error),
        })
    })
    .unwrap_or_else(|report| report)
}

/// The --with form, inside the lock: each name read by resolve_squash in order (the first refusal is the answer),
/// then land's would_merge_with of the squash and them into the config's integration branch, reported by
/// would_merge_with_report or, when refused, land_error_report. It carries no savings.
fn with_answer(git: &Git, config: &Config, squash: &Squash, names: &[SquashName]) -> Result<Report, Report> {
    let with = names.iter().map(|name| resolve_squash(git, VERB, name)).collect::<Result<Vec<_>, _>>()?;
    Ok(match land::would_merge_with(git, &guard_refs_for(config), squash, &with) {
        Ok(answer) => would_merge_with_report(squash, &with, &answer),
        Err(error) => land_error_report(VERB, &error, &config.integration_branch),
    })
}
