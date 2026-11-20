//! run_land: `mr land <commit>`, a reviewed squash landed on integration under the land lock, and the
//! integration folder moved with integration (land land). --key names the lock holder only, never passed
//! to the op log; repeat safety comes from land's answers (a repeat of a landed squash answers Landed) and
//! from the stopped-move finish: before land's call, finish_stopped_move moves an integration folder a stopped run
//! left on the old commit onto integration's tip.
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_land_args_tests,
//! land_report_tests, note_finished_move_tests and landing's land_error_report_tests cover its decisions.
//! The integration tests in crates/cli/tests/land_runs/land_flow/ cover it through the binary: landing.rs land by
//! fast-forward and by merge commit, with and without an integration folder (the folder moved from the origin tip to
//! the landed commit), conflicted (exit 2 with the files and integration unmoved), busy (exit 3 naming the other
//! change), a repeat after Landed answering landed and moving nothing more, a change ID word (exit 4), the land lock
//! held by a live pid (exit 3) and without a config (exit 1, next mr init); killed.rs the stopped-move finish (a
//! kill after land's op and before the folder move, then a plain repeat moves the folder and the report's first
//! line says so); stopped.rs a staged edit in the integration folder left untouched before land's call.
//! Integration tests to write (not written here): the acceptance checks of land.
//! A refused folder move is finished by a repeat when the folder was cleared as the refusal says (paths moved out or
//! put back to the index copy, the index left as it is); a clear that resets the index to HEAD (the folder's HEAD is
//! already the branch's new commit) leaves no stall for the repeat to see, and the folder's files are then their
//! owner's to bring to HEAD, unreported.

use std::path::Path;

use ::land::{LandAnswer, LandRefs};
use gitio::{Git, Oid};
use oplog::{KillPidProbe, LockName};

use super::{land_report, parse_land_args};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::integration_folder::{finish_stopped_move, move_integration_folder, note_finished_move};
use crate::landing::{land_error_report, resolve_squash, SquashName};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "land";

/// The commit read by parse_land_args (an Err is land's usage report, exit 4, before the verb runs git). Then, under
/// the land lock after recover (locked_act), keyed by --key or op_key("land", the common dir): first a stopped
/// integration folder move is finished (finish_stopped_move), then the squash at that commit read (resolve_squash),
/// the origin tip read, and land's land as the context's actor (refused: land_error_report). On Landed the
/// integration folder is moved from the origin tip to the landed commit (move_integration_folder); Conflicted and
/// Busy move no folder. land_report reports it. A finished stopped move is the first line of every report after it,
/// refusals included (note_finished_move).
pub fn run_land(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let commit = match parse_land_args(&args.rest) {
        Ok(commit) => commit,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before land runs");
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let refs = guard_refs_for(config);
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    let repeat = format!("mr land {commit} --actor <name>");
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let finished = finish_stopped_move(&git, &common, VERB, &refs.integration_branch, &context.actor, &repeat)?;
        land(&git, &refs, commit, &context.actor, &repeat)
            .map(|report| note_finished_move(report, finished.as_ref()))
            .map_err(|report| note_finished_move(report, finished.as_ref()))
    })
    .unwrap_or_else(|report| report)
}

/// The squash at `commit` read, the origin tip read, land's land, and on Landed the folder moved with integration;
/// `repeat` ends a refused move's next.
fn land(git: &Git, refs: &LandRefs, commit: Oid, actor: &str, repeat: &str) -> Result<Report, Report> {
    let squash = resolve_squash(git, VERB, &SquashName::Commit(commit))?;
    let origin_branch = &refs.origin_branch;
    let Ok(Some(origin)) = git.read_ref(&format!("refs/heads/{origin_branch}")) else {
        return Err(failure_report(VERB, &format!("git could not read the origin branch {origin_branch}")));
    };
    let integration = &refs.integration_branch;
    let answer =
        ::land::land(git, refs, &squash, actor).map_err(|error| land_error_report(VERB, &error, integration))?;
    let LandAnswer::Landed { integration: landed } = &answer else {
        return Ok(land_report(&squash, &answer, None));
    };
    let done = format!("landed change {}: integration is at {landed}", squash.change_id);
    let folder = move_integration_folder(git, VERB, integration, &origin, landed, actor, &done, repeat)?;
    Ok(land_report(&squash, &answer, folder.as_deref()))
}
