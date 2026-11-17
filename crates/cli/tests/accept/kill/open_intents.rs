//! The intents an op log still holds open.

use super::OpLine;

/// Each "intent" record with no later "done" or "failed" record of the same op_id, in log order.
pub fn open_intents(lines: &[OpLine]) -> Vec<&OpLine> {
    lines
        .iter()
        .enumerate()
        .filter(|(at, line)| {
            line.phase == "intent"
                && !lines[at + 1..].iter().any(|later| later.op_id == line.op_id && later.phase != "intent")
        })
        .map(|(_, line)| line)
        .collect()
}
