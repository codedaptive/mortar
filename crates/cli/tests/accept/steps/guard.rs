//! Step 2, the guard (02-guard.md): its body is crate::guard::run_guard, the set-up with a squash ready to land and
//! the nine checks in order.

use super::{Gate, StepSpec};

pub const GUARD: StepSpec =
    StepSpec { number: 2, title: "the guard", order: "ACCEPT-2", gate: Gate::Always, body: Some(crate::guard::run_guard) };
