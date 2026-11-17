//! run_publish: `mortar publish`, the landing on integration pushed to the remote's origin branch under the land lock
//! (SPEC-cli 06's row: land publish; exit 1 when origin moved, saying land again). On OriginMoved the integration
//! folder moves with integration onto the fetched origin, as back-out moves it (Coord-cli ruling for WO-cli-7b; 06's
//! publish row names only land publish). --key names the lock holder only, never passed to the op
//! log (S-132); repeat safety comes from land's answers (a repeat settles a push intent, S-103).
//! KNOWN GAP: a run stopped after land's publish answered OriginMoved and before the folder moved leaves the folder's
//! files on the old commit, and a repeat answers NothingToPublish and moves no folder, since the landing state is gone.
//! No sibling test (S-104): it takes a real lock and runs git, the op log and land; publish_report_tests and
//! landing's publish_error_report_tests cover its decisions. The RIT crates/cli/tests/cli_rit/run_publish_tests.rs covers
//! it through the binary.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119): the land lock held by a live pid, exit
//! 3; without a config, exit 1, next mortar init; the PublishError texts through a real remote (PushRefused by a
//! remote hook, RemoteMissing, a PushIntent left open); ACCEPT-1 checks 12 and 13, ACCEPT-2 check 8 (with mortar
//! land, once built).

use std::path::Path;

use gitio::Git;
use land::PublishAnswer;
use oplog::{KillPidProbe, LockName};

use super::publish_report;
use crate::args::no_arguments_report;
use crate::guard::guard_refs_for;
use crate::integration_folder::move_integration_folder;
use crate::landing::publish_error_report;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "publish";

/// Any argument is publish's usage report, exit 4, before any git. Then, under the land lock after recover
/// (locked_act), keyed by --key or op_key("publish", the common dir): integration's tip read before, land's publish
/// as the context's actor, reported by publish_report or, when refused, publish_error_report. On OriginMoved the
/// integration folder is moved from that tip onto the found origin first; a refused move is its own report.
pub fn run_publish(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    if let Some(report) = no_arguments_report(VERB, &args.rest, args.json) {
        return report;
    }
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before publish runs");
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let refs = guard_refs_for(config);
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let integration = &config.integration_branch;
        let Ok(Some(before)) = git.read_ref(&format!("refs/heads/{integration}")) else {
            return Err(failure_report(VERB, &format!("git could not read the integration branch {integration}")));
        };
        let answer = land::publish(&git, &refs, &context.actor).map_err(|error| publish_error_report(&error, &refs))?;
        let found = match &answer {
            PublishAnswer::OriginMoved { found } => found,
            PublishAnswer::Published { .. } | PublishAnswer::NothingToPublish => {
                return Ok(publish_report(&answer, &refs, None));
            }
        };
        let done = format!(
            "someone else moved {}'s {} to {found}; integration is back on it",
            refs.remote, refs.origin_branch
        );
        let folder = move_integration_folder(&git, VERB, integration, &before, found, &context.actor, &done)?;
        Ok(publish_report(&answer, &refs, folder.as_deref()))
    })
    .unwrap_or_else(|report| report)
}
