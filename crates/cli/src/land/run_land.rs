//! run_land: `mr land <commit>`, a reviewed squash landed on integration under the land lock as a member of the
//! landing batch (land land_batch), and the integration folder moved with integration. --key names the lock holder
//! only, never passed to the op log; repeat safety comes from land's answers (a repeat of a landed squash answers
//! Landed) and from the stopped-move finish: before land's call, finish_stopped_move moves an integration folder a
//! stopped run left on the old commit onto integration's tip.
//! The folder moves from the old value of the last op that moved integration (for a fresh landing or an appended
//! member, integration's tip before this call) to the tip. The folder's HEAD is never the measure: it is the branch's
//! tip once land moves the ref. A repeat re-runs that move: a no-op naming the folder at the tip when it is there, and
//! a move, or its refusal with the paths, when a refused move left it behind.
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_land_args_tests,
//! land_report_tests, note_finished_move_tests and landing's land_error_report_tests cover its decisions.
//! The integration tests in crates/cli/tests/land_runs/land_flow/ cover it through the binary: landing.rs land by
//! fast-forward and by merge commit, with and without an integration folder (the folder moved from integration's tip
//! before the landing to the one after it), conflicted (exit 2 with the files and integration unmoved), another
//! zone's squash landing in the batch, a repeat after Landed answering landed with no further op, a change ID
//! word (exit 4), the land lock held by a live pid (exit 3) and without a config (exit 1, next mr init); batch.rs two
//! zones landing as one batch in order, the second member's edit merged into the integration folder beside the
//! first's, a repeat of the first member reporting the batch's tip and the folder line naming the folder at the tip
//! with no further op, a batch of one's repeat keeping its two text lines with no further op, a second squash of a
//! zone already in the batch refused (zone-in-batch), and a batch of one keeping the single-landing text; killed.rs
//! the stopped-move finish (a kill after land's op and before the folder move, then a plain repeat moves the folder
//! and the report's first line says so); stopped.rs a staged edit in the integration folder left untouched while the
//! peer's squash lands, a repeat of land refusing a staged edit to a moved path (with the paths) until it is cleared,
//! and a repeat refusing an unmerged folder index with integration, log and state unchanged.
//! Integration tests to write (not written here): the acceptance checks of land.
//! A refused folder move is finished by a repeat when the folder was cleared as the refusal says (paths moved out or
//! put back to the index copy, the index left as it is); a clear that resets the index to HEAD (the folder's HEAD is
//! already the branch's new commit) leaves no stall for the repeat to see, and the folder's files are then their
//! owner's to bring to HEAD, unreported.

use std::path::Path;

use ::land::{BatchLandAnswer, LandRefs};
use gitio::{Git, Oid};
use oplog::{read_op_log, KillPidProbe, LockName};

use super::{land_report, parse_land_args};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::integration_folder::{
    finish_stopped_move, integration_tip, last_integration_old, move_integration_folder, note_finished_move,
    stopped_before_move,
};
use crate::landing::{land_error_report, read_batch, resolve_squash, SquashName};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "land";

/// The commit read by parse_land_args (an Err is land's usage report, exit 4, before the verb runs git). Then, under
/// the land lock after recover (locked_act), keyed by --key or op_key("land", the common dir): first a stopped
/// integration folder move is finished (finish_stopped_move), then the squash at that commit read (resolve_squash),
/// integration's tip read (`before`), and land's land_batch as the context's actor (refused: land_error_report). On
/// Landed integration's tip is read again (`after`) and reported as integration's tip: for a repeat of an earlier
/// member land answers that member's commit, while integration stands at the batch's tip. The integration folder is
/// moved to `after` (move_integration_folder) from the old value of the last op that moved integration
/// (last_integration_old), or from `before` when the op log holds no such op or its old is `after`; for a repeat that
/// is the same move again. Conflicted, Busy and ZoneInBatch move no folder. After Landed (before the move) and
/// Conflicted the landing batch is read (read_batch, no state an empty batch) and land_report reports it all. After
/// Landed, every failed read before the move (the second tip read, the op log, the batch) is exit 1
/// "folder-not-moved" led by the landed line (stopped_before_move), repeat as next; when the tip read is what failed,
/// that line names the change and its commit without the tip. A finished stopped move is the first line of every
/// report after it, refusals included (note_finished_move).
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
        land(&git, &common, &refs, commit, &context.actor, &repeat)
            .map(|report| note_finished_move(report, finished.as_ref()))
            .map_err(|report| note_finished_move(report, finished.as_ref()))
    })
    .unwrap_or_else(|report| report)
}

/// The squash at `commit` read, integration's tip read before land's land_batch, the landing batch read after
/// Conflicted (its error the bare report); on Landed integration's tip read again, the landed line built, and then
/// the op log in `common` and the landing batch read. Each of those three reads' errors is a report led by the
/// landed line (stopped_before_move), the line without the tip when the tip read failed; `repeat` ends their next
/// and a refused move's. The folder moves from the last integration op's old value (a fresh landing or an appended
/// member: the tip before this call) to the tip; a repeat re-runs that move, a no-op naming the folder at the tip
/// when it is there, and a move or its refusal when a refused move left it behind.
fn land(git: &Git, common: &Path, refs: &LandRefs, commit: Oid, actor: &str, repeat: &str) -> Result<Report, Report> {
    let squash = resolve_squash(git, VERB, &SquashName::Commit(commit))?;
    let integration = &refs.integration_branch;
    let before = integration_tip(git, VERB, integration)?;
    let answer =
        ::land::land_batch(git, refs, &squash, actor).map_err(|error| land_error_report(VERB, &error, integration))?;
    match &answer {
        BatchLandAnswer::Landed { .. } => {}
        BatchLandAnswer::Conflicted { .. } => return Ok(land_report(&squash, &answer, &read_batch(git, VERB)?, None)),
        BatchLandAnswer::Busy { .. } | BatchLandAnswer::ZoneInBatch { .. } => {
            return Ok(land_report(&squash, &answer, &[], None))
        }
    }
    let after = integration_tip(git, VERB, integration).map_err(|error| {
        stopped_before_move(&format!("landed change {} (commit {})", squash.change_id, squash.commit), error, repeat)
    })?;
    let done = format!("landed change {}: integration is at {after}", squash.change_id);
    let log = read_op_log(common).map_err(|_| {
        stopped_before_move(&done, failure_report(VERB, "the op log could not be read"), repeat)
    })?;
    // An old equal to `after` is replaced by `before`: move_integration_folder answers Ok(None) ("no integration
    // folder to move") when old == new, which would be false when a folder exists; old == after happens only when
    // the log's last integration op is a move of integration back to that op's old value that the op log does not
    // hold (last_integration_old skips moves that leave the ref where it was).
    let old = last_integration_old(&log.records, &format!("refs/heads/{integration}"))
        .filter(|old| *old != after)
        .unwrap_or(before);
    let batch = read_batch(git, VERB).map_err(|error| stopped_before_move(&done, error, repeat))?;
    let folder = move_integration_folder(git, VERB, integration, &old, &after, actor, &done, repeat)?;
    Ok(land_report(&squash, &BatchLandAnswer::Landed { integration: after }, &batch, folder.as_deref()))
}
