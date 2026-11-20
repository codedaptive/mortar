//! Runs `mr` under the kill shim and proves the armed transaction was the case's own. Real processes and a real
//! SIGKILL: every case of run_kill.rs runs through it on every drive.

use std::path::Path;

use super::{open_intents, read_log, CaseLines, OpLine};
use crate::report::Checks;
use crate::world::{Crash, World};

/// `mr <args>` in `cwd`, killed at its first `git -C <dir> update-ref` (`crash` before or after it), then the op
/// log read. The case's next line holds when the run ended by signal 9 and the log holds exactly one open intent,
/// of kind `kind`; each open intent is a log line. The killed op and the log as the kill left it, or None when
/// that line failed.
pub fn kill_armed(
    world: &World,
    checks: &mut Checks,
    lines: &mut CaseLines,
    (cwd, args): (&Path, &[&str]),
    crash: Crash,
    kind: &str,
) -> Option<(OpLine, Vec<OpLine>)> {
    let ran = match world.killed(cwd, World::mr_bin(), args, crash) {
        Ok(ran) => ran,
        Err(error) => {
            lines.check(checks, false, &format!("the kill path failed: {error}"));
            return None;
        }
    };
    let log = match read_log(world) {
        Ok(log) => log,
        Err(error) => {
            lines.check(checks, false, &format!("{}; {error}", ran.seen()));
            return None;
        }
    };
    let open: Vec<OpLine> = open_intents(&log).into_iter().cloned().collect();
    let kinds: Vec<&str> = open.iter().map(|op| op.kind.as_str()).collect();
    let saw = format!("{}; open intents [{}], expected one {kind}", ran.seen(), kinds.join(", "));
    let ok = ran.signal == Some(9) && kinds == [kind];
    for op in &open {
        checks.log(&format!("open after the kill: {}", op.shown()));
    }
    if !lines.check(checks, ok, &saw) {
        return None;
    }
    open.into_iter().next().map(|killed| (killed, log))
}
