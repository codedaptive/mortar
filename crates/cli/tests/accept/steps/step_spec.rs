//! A Done step as the drive lists it. Data only: decide_step_tests.rs covers how it is read.

use super::Gate;
use crate::report::Checks;
use crate::world::World;

/// A step's checks, run in its own fresh world; each check records one line.
pub type StepBody = fn(&World, &mut Checks);

/// The step's number and title in the report, the order that writes its body, its gate, and its body (None until
/// that order fills it in).
#[derive(Debug, Clone, Copy)]
pub struct StepSpec {
    pub number: u8,
    pub title: &'static str,
    pub order: &'static str,
    pub gate: Gate,
    pub body: Option<StepBody>,
}
