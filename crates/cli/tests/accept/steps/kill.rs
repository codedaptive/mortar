//! Step 3, nothing lost, nothing twice (03-kill.md): its body is crate::kill::run_kill, the set-up, the four cases
//! (case 3 killed before and after) each killing `mr` with World::killed, and the consistency items after each.

use super::{Gate, StepSpec};

pub const KILL: StepSpec = StepSpec {
    number: 3,
    title: "nothing lost, nothing twice",
    order: "ACCEPT-3",
    gate: Gate::Always,
    body: Some(crate::kill::run_kill),
};
