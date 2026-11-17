//! run_back_out: `mortar back-out`, integration moved back to where the landing found it and the integration folder
//! moved back with it, under the land lock (SPEC-cli 06: land back_out, integration folder moved back).
//! --key names the lock holder only, never passed to the op log (S-132); repeat safety comes from
//! land's answers (NothingToBackOut).
//! KNOWN GAP: a run stopped after land moved integration back and before the folder moved leaves the folder's files
//! on the landed commit, and a repeat answers NothingToBackOut and moves no folder, since land's answer does not
//! carry the old tip.
//! No sibling test (S-104): it takes a real lock and runs git, the op log and land; back_out_report_tests and
//! landing's land_error_report_tests cover its decisions. The RIT crates/cli/tests/cli_rit/run_back_out_tests.rs covers
//! it through the binary.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119): the land lock held by a live pid,
//! exit 3; without a config, exit 1, next mortar init; the LandError texts that need git, the op log or the landing
//! state to fail for real; ACCEPT-1 check 11.

use std::path::Path;

use gitio::{Git, Oid};
use land::BackOutAnswer;
use oplog::{KillPidProbe, LockName};

use super::back_out_report;
use crate::args::no_arguments_report;
use crate::guard::guard_refs_for;
use crate::integration_folder::move_integration_folder;
use crate::landing::land_error_report;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "back-out";

/// Any argument is back-out's usage report, before any git. Then, under the land lock after recover (locked_act),
/// keyed by --key or op_key("back-out", the common dir): integration's tip read, land's back_out as the context's
/// actor (refused: land_error_report), and on BackedOut the integration folder moved from the old tip to the new
/// one (move_integration_folder), reported by back_out_report. NothingToBackOut moves no folder.
pub fn run_back_out(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    if let Some(usage) = no_arguments_report(VERB, &args.rest, args.json) {
        return usage;
    }
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before back-out runs");
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let refs = guard_refs_for(config);
    let integration = &config.integration_branch;
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let before = integration_tip(&git, integration)?;
        let answer = land::back_out(&git, &refs, &context.actor)
            .map_err(|error| land_error_report(VERB, &error, integration))?;
        let BackOutAnswer::BackedOut { change_id } = &answer else {
            return Ok(back_out_report(&answer, &before, None));
        };
        let after = integration_tip(&git, integration)?;
        let done = format!("backed out change {change_id}: integration is back at {after}");
        let folder = move_integration_folder(&git, VERB, integration, &before, &after, &context.actor, &done)?;
        Ok(back_out_report(&answer, &after, folder.as_deref()))
    })
    .unwrap_or_else(|report| report)
}

/// refs/heads/<integration>'s tip; git failing or the branch missing is failure_report in the cli's own words.
fn integration_tip(git: &Git, integration: &str) -> Result<Oid, Report> {
    match git.read_ref(&format!("refs/heads/{integration}")) {
        Ok(Some(tip)) => Ok(tip),
        Ok(None) => Err(failure_report(VERB, &format!("refs/heads/{integration} does not exist"))),
        Err(_) => Err(failure_report(VERB, &format!("git could not read refs/heads/{integration}"))),
    }
}
