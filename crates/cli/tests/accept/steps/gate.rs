//! When a Done step may run. Data only: decide_step_tests.rs covers how it is read.

/// Always, or only when an environment variable is set and not empty; `meaning` says what to set it to.
#[derive(Debug, Clone, Copy)]
pub enum Gate {
    Always,
    Env { var: &'static str, meaning: &'static str },
}
