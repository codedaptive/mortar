//! Step 2, the guard (02-guard.md). ACCEPT-2 writes its body here and sets `body: Some(<its fn>)`; until then the
//! report shows it NOT RUN.

use super::{Gate, StepSpec};

pub const GUARD: StepSpec = StepSpec { number: 2, title: "the guard", order: "ACCEPT-2", gate: Gate::Always, body: None };
