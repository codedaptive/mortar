//! A Done step as the drive lists it. Data only: decide_step_tests.rs covers how it is read.

use super::{Gate, StepBody};

/// The step's number and title in the report, the acceptance item its body is written for, its gate, and its body
/// (None until it is written).
#[derive(Debug, Clone, Copy)]
pub struct StepSpec {
    pub number: u8,
    pub title: &'static str,
    pub order: &'static str,
    pub gate: Gate,
    pub body: Option<StepBody>,
}
