//! A case's "the log says what happened" line: how the killed op was closed, and how many done ops of its kind moved
//! its ref since. Real processes: cases 1-3 of run_kill.rs run it on every drive.

use super::{done_moves_of, read_log, CaseLines, OpLine};
use crate::report::Checks;
use crate::world::World;

/// Reads the op log. The line holds when the killed op's closing record has phase and result `closed` (for a join or
/// land op, recover is the only writer of a not-moved result, and the only one that closes another process's intent before the repeat
/// runs) and exactly one done op of the killed op's kind has moved `refname` since the kill (done_moves_of).
/// Every record of the killed op and each such done op is a log line. True when the line held.
pub fn kill_log_check(
    world: &World,
    checks: &mut Checks,
    lines: &mut CaseLines,
    killed: &OpLine,
    closed: (&str, &str),
    refname: &str,
) -> bool {
    let log = match read_log(world) {
        Ok(log) => log,
        Err(error) => return lines.check(checks, false, &error),
    };
    for op in log.iter().filter(|op| op.op_id == killed.op_id) {
        checks.log(&format!("the killed op: {}", op.shown()));
    }
    let done: Vec<&OpLine> =
        done_moves_of(&log, refname, &killed.op_id).into_iter().filter(|op| op.kind == killed.kind).collect();
    for op in &done {
        checks.log(&format!("done since the kill: {}", op.shown()));
    }
    let closing = log.iter().find(|op| op.op_id == killed.op_id && op.phase != "intent");
    let closing_text = closing.map_or("not closed".to_string(), |op| format!("closed {} ({})", op.phase, op.result));
    let ok = closing.is_some_and(|op| (op.phase.as_str(), op.result.as_str()) == closed) && done.len() == 1;
    let saw = format!(
        "the killed op {} {closing_text}, expected {} ({}); done {} ops moving {refname} since the kill: {}",
        killed.op_id,
        closed.0,
        closed.1,
        killed.kind,
        done.len()
    );
    lines.check(checks, ok, &saw)
}
