//! run_back_out: `mr back-out`, integration moved back to where the landing batch found it (every member of the
//! batch goes) and the integration folder moved back with it, under the land lock (land's back_out_batch, then the
//! integration folder moved back).
//! --key names the lock holder only, never passed to the op log; repeat safety comes from
//! land's answers (NothingToBackOut) and from the stopped-move finish: before land's call, finish_stopped_move moves
//! an integration folder a stopped run left on the old commit onto integration's tip, so a plain repeat after
//! a kill finishes the move even when land answers NothingToBackOut. A refused folder move is finished by a repeat
//! when the folder was cleared as the refusal says (paths moved out or put back to the index copy, the index left as
//! it is); a clear that resets the index to HEAD (the folder's HEAD is already the branch's new commit) leaves no
//! stall for the repeat to see, and the folder's files are then their owner's to bring to HEAD, unreported.
//! No sibling test: it takes a real lock and runs git, the op log and land; back_out_report_tests,
//! note_finished_move_tests and landing's land_error_report_tests cover its decisions. The integration test
//! crates/cli/tests/land_runs/run_back_out_tests.rs covers it through the binary;
//! crates/cli/tests/land_runs/land_flow/landing.rs covers the land lock held by a live pid (exit 3),
//! land_flow/killed.rs the stopped-move finish (a kill after land's op and before the folder move, then a plain
//! repeat moves the folder and the report's first line says so) and land_flow/stopped.rs an owner's edit to a path
//! the move changes refusing back-out's own folder move (folder-not-moved, the index left as it is), and a staged
//! edit left alone while land and publish go on.
//! Integration tests to write (not written here): without a config, exit 1, next mr init (land_flow/landing.rs
//! covers it for land and would-merge only); the LandError texts that need git, the op log or the landing
//! state to fail for real; a back-out after a failed integration test of a landed squash.

use std::path::Path;

use gitio::{Git, Oid};
use land::{BatchBackOutAnswer, LandRefs};
use oplog::{KillPidProbe, LockName};

use super::back_out_report;
use crate::args::no_arguments_report;
use crate::guard::guard_refs_for;
use crate::integration_folder::{finish_stopped_move, move_integration_folder, note_finished_move, stopped_before_move};
use crate::landing::land_error_report;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "back-out";
/// The same command run again, which a refused integration folder move's next ends with.
const REPEAT: &str = "mr back-out --actor <name>";

/// Any argument is back-out's usage report, before the verb runs git. Then, under the land lock after recover
/// (locked_act), keyed by --key or op_key("back-out", the common dir): first a stopped integration folder move is
/// finished (finish_stopped_move), then integration's tip read, land's back_out_batch as the context's actor (refused:
/// land_error_report), and on BackedOut the integration folder moved from the old tip to the new one
/// (move_integration_folder), reported by back_out_report. After BackedOut, a failed tip read is exit 1
/// "folder-not-moved" led by the backed-out line without the tip, repeat as next. NothingToBackOut moves no folder of
/// its own; a finished stopped move is the first line of every report after it, refusals included
/// (note_finished_move).
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
        let finished = finish_stopped_move(&git, &common, VERB, integration, &context.actor, REPEAT)?;
        back_out(&git, &refs, integration, &context.actor)
            .map(|report| note_finished_move(report, finished.as_ref()))
            .map_err(|report| note_finished_move(report, finished.as_ref()))
    })
    .unwrap_or_else(|report| report)
}

/// integration's tip read, land's back_out_batch, and on BackedOut integration's tip read again and the folder moved
/// back with integration. The move's done text, the first line of a refused move's report, is back_out_report's
/// first line, so it names every member of the batch in landing order; a failed second tip read is led by that line
/// without the tip (stopped_before_move).
fn back_out(git: &Git, refs: &LandRefs, integration: &str, actor: &str) -> Result<Report, Report> {
    let before = integration_tip(git, integration)?;
    let answer = land::back_out_batch(git, refs, actor).map_err(|error| land_error_report(VERB, &error, integration))?;
    let BatchBackOutAnswer::BackedOut { change_id, change_ids } = &answer else {
        return Ok(back_out_report(&answer, &before, None));
    };
    let after = integration_tip(git, integration).map_err(|error| {
        let line = if change_ids.len() <= 1 {
            format!("backed out change {change_id}")
        } else {
            let ids = change_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            format!("backed out changes {ids} (in landing order)")
        };
        stopped_before_move(&line, error, REPEAT)
    })?;
    let done = back_out_report(&answer, &after, None).lines.remove(0);
    let folder = move_integration_folder(git, VERB, integration, &before, &after, actor, &done, REPEAT)?;
    Ok(back_out_report(&answer, &after, folder.as_deref()))
}

/// refs/heads/<integration>'s tip; git failing or the branch missing is failure_report in the cli's own words.
fn integration_tip(git: &Git, integration: &str) -> Result<Oid, Report> {
    match git.read_ref(&format!("refs/heads/{integration}")) {
        Ok(Some(tip)) => Ok(tip),
        Ok(None) => Err(failure_report(VERB, &format!("refs/heads/{integration} does not exist"))),
        Err(_) => Err(failure_report(VERB, &format!("git could not read refs/heads/{integration}"))),
    }
}
