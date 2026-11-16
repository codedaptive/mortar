//! Whether a step runs. Data only: decide_step_tests.rs covers how it is decided.

use super::StepBody;

/// Run the body, or do not, with the reason the report shows.
#[derive(Debug)]
pub enum Decision {
    Run(StepBody),
    NotRun(String),
}
