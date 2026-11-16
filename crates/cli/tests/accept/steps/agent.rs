//! Step 4, the agent test (04-agent.md). ACCEPT-4 writes its body here and sets `body: Some(<its fn>)`; the step
//! runs only when ACCEPT_AGENT is set.

use super::{Gate, StepSpec};

pub const AGENT: StepSpec = StepSpec {
    number: 4,
    title: "the agent test",
    order: "ACCEPT-4",
    gate: Gate::Env { var: "ACCEPT_AGENT", meaning: "the command that runs one agent turn loop (04-agent.md)" },
    body: None,
};
