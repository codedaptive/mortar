//! Step 3, nothing lost, nothing twice (03-kill.md). ACCEPT-3 writes its body here, killing mortar with
//! World::killed and World::mortar_bin, and sets `body: Some(<its fn>)`; until then the report shows it NOT RUN.

use super::{Gate, StepSpec};

pub const KILL: StepSpec =
    StepSpec { number: 3, title: "nothing lost, nothing twice", order: "ACCEPT-3", gate: Gate::Always, body: None };
