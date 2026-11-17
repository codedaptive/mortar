//! Step 1, the flow (01-flow.md): its body is crate::flow::run_flow, the set-up and the fifteen checks in order.

use super::{Gate, StepSpec};

pub const FLOW: StepSpec =
    StepSpec { number: 1, title: "the flow", order: "ACCEPT-1", gate: Gate::Always, body: Some(crate::flow::run_flow) };
