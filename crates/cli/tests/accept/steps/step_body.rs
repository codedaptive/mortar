//! A step's body. Data only: run_step_tests.rs runs bodies through run_step, and decide_step_tests.rs covers how
//! a spec's body decides whether the step runs.

use crate::report::Checks;
use crate::world::World;

/// A step's checks, run in its own fresh world; each check records one line.
pub type StepBody = fn(&World, &mut Checks);
