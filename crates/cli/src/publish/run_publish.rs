//! run_publish: `mr publish [--no-push]`, the landing on integration put on the local origin branch under the land
//! lock, pushed to the remote's origin branch first unless --no-push (land publish_with_push; exit 3 when origin
//! moved, saying land again). With --no-push the remote is never read and nothing is fetched; origin moved means the
//! local origin branch moved. On OriginMoved the integration folder moves with integration onto the found origin, as
//! back-out moves it. --key names the lock holder only, never passed to the op log; repeat safety comes from land's
//! answers (a push-on repeat settles a push intent) and from the stopped-move finish: before land's call,
//! finish_stopped_move moves an integration folder a stopped run left on the old commit onto integration's tip, so a
//! plain repeat after a kill finishes the move even when land answers NothingToPublish. A refused folder move is
//! finished by a repeat when the folder was cleared as the refusal says (paths moved out or put back to the index
//! copy, the index left as it is); a clear that resets the index to HEAD (the folder's HEAD is already the branch's
//! new commit) leaves no stall for the repeat to see, and the folder's files are then their owner's to bring to HEAD,
//! unreported. The repeat a refused move names keeps --no-push when it was given.
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_publish_args_tests,
//! publish_report_tests, note_finished_move_tests and landing's publish_error_report_tests cover its decisions. The
//! integration tests crates/cli/tests/land_runs/run_publish_tests.rs and run_publish_no_push_tests.rs cover it
//! through the binary; crates/cli/tests/land_runs/land_flow/landing.rs covers the land lock held by a live pid
//! (exit 4), land_flow/killed.rs the stopped-move finish (a kill after land's op and before the folder move, then a
//! plain repeat moves the folder from the op's old value to the tip and the report's first line says so) and
//! land_flow/stopped.rs a staged edit in the integration folder (no move, the verb goes on); land_flow/batch.rs the
//! next naming `mr after-publish` for each zone of a batch, in landing order.
//! Integration tests to write (not written here): without a config, exit 2, next mr init (land_flow/landing.rs
//! covers it for land and would-merge only); the PublishError texts through a real remote (PushRefused by a
//! remote hook, RemoteMissing, a PushIntent already done).

use std::path::Path;

use gitio::Git;
use land::{LandRefs, PublishAnswer};
use oplog::{KillPidProbe, LockName};

use super::{parse_publish_args, publish_report};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::integration_folder::{finish_stopped_move, integration_tip, move_integration_folder, note_finished_move};
use crate::landing::{publish_error_report, read_batch};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "publish";
/// The same command run again, which a refused integration folder move's next ends with: push on.
const REPEAT: &str = "mr publish --actor <name>";
/// The same under --no-push.
const REPEAT_NO_PUSH: &str = "mr publish --no-push --actor <name>";

/// Words parse_publish_args refuses are publish's usage report, exit 2, before the verb runs git. Then, under the land
/// lock after recover (locked_act), keyed by --key or op_key("publish", the common dir): first a stopped integration
/// folder move is finished (finish_stopped_move), then integration's tip and the landing batch's zones read, land's
/// publish_with_push as the context's actor with push off under --no-push, reported by publish_report or, when
/// refused, publish_error_report. On OriginMoved the integration folder is moved from that tip onto the found origin
/// first; a refused move is its own report. A finished stopped move is the first line of every report after it,
/// refusals included (note_finished_move).
pub fn run_publish(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let push = match parse_publish_args(&args.rest) {
        Ok(push) => push,
        Err(reason) => return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json }),
    };
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
    let repeat = if push { REPEAT } else { REPEAT_NO_PUSH };
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let finished = finish_stopped_move(&git, &common, VERB, &config.integration_branch, &context.actor, repeat)?;
        publish(&git, &refs, &context.actor, push, repeat)
            .map(|report| note_finished_move(report, finished.as_ref()))
            .map_err(|report| note_finished_move(report, finished.as_ref()))
    })
    .unwrap_or_else(|report| report)
}

/// integration's tip read (integration_tip), then the batch's zones in landing order (read_batch, before
/// publish_with_push removes the landing state; no state is no zones), land's publish_with_push, and on OriginMoved
/// the folder moved with integration onto `found`, its done line naming the remote's origin branch under push on and
/// the local one under push off.
fn publish(git: &Git, refs: &LandRefs, actor: &str, push: bool, repeat: &str) -> Result<Report, Report> {
    let integration = &refs.integration_branch;
    let before = integration_tip(git, VERB, integration)?;
    let zones: Vec<String> = read_batch(git, VERB)?.into_iter().map(|member| member.zone).collect();
    let outcome = land::publish_with_push(git, refs, actor, push).map_err(|error| publish_error_report(&error, refs))?;
    let found = match &outcome.answer {
        PublishAnswer::OriginMoved { found } => found,
        PublishAnswer::Published { .. } | PublishAnswer::NothingToPublish => {
            return Ok(publish_report(&outcome.answer, outcome.pushed, push, refs, &zones, None));
        }
    };
    let origin_branch = &refs.origin_branch;
    let done = if push {
        format!("someone else moved {}'s {origin_branch} to {found}; integration is back on it", refs.remote)
    } else {
        format!("the local {origin_branch} was moved to {found}; integration is back on it")
    };
    let folder = move_integration_folder(git, VERB, integration, &before, found, actor, &done, repeat)?;
    Ok(publish_report(&outcome.answer, outcome.pushed, push, refs, &zones, folder.as_deref()))
}
