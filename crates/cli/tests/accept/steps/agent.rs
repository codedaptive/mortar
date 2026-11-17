//! Step 4, the agent test (04-agent.md): its body is crate::agent::run_agent, the set-up, the agent run through the
//! logging wrappers with a conflict joined in just before its first join, the conflict line and checks 1-4. It runs only
//! when ACCEPT_AGENT is set.

use super::{Gate, StepSpec};

pub const AGENT: StepSpec = StepSpec {
    number: 4,
    title: "the agent test",
    order: "ACCEPT-4",
    gate: Gate::Env { var: "ACCEPT_AGENT", meaning: "the command that runs one agent turn loop (04-agent.md)" },
    body: Some(crate::agent::run_agent),
};
