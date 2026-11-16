//! Step 1, the flow (01-flow.md). ACCEPT-1 writes its body here and sets `body: Some(<its fn>)`; until then the
//! report shows it NOT RUN.

use super::{Gate, StepSpec};

pub const FLOW: StepSpec = StepSpec { number: 1, title: "the flow", order: "ACCEPT-1", gate: Gate::Always, body: None };
