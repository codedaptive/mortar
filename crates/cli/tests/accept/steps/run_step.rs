//! Runs one Done step into its report block. Real repositories and processes: the drive (main.rs, `done_list`) runs
//! every step through it.

use std::any::Any;
use std::ffi::OsString;
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::{decide_step, Decision, StepSpec};
use crate::report::{block_verdict, Block, Checks, Verdict};
use crate::world::World;

/// A step that does not run is a NOT RUN block with no lines. One that runs gets a fresh world; a world that
/// cannot be made fails the step. The body runs with its panic caught, so one step's panic is a FAIL line in its
/// own block and the next steps still run; then the world is removed, recorded as one more check.
pub fn run_step(spec: &StepSpec, env: &dyn Fn(&str) -> Option<OsString>) -> Block {
    let block = |verdict, lines| Block { number: spec.number, title: spec.title, verdict, lines };
    let body = match decide_step(spec, env) {
        Decision::NotRun(reason) => return block(Verdict::NotRun(reason), Vec::new()),
        Decision::Run(body) => body,
    };
    let mut checks = Checks::new();
    let world = match World::new() {
        Ok(world) => world,
        Err(error) => {
            checks.check("made the throwaway repositories", false, &error.to_string());
            return block(Verdict::Fail, checks.lines());
        }
    };
    let panic = catch_unwind(AssertUnwindSafe(|| body(&world, &mut checks))).err().map(|payload| panic_text(&*payload));
    if let Some(text) = &panic {
        checks.check("the step ran to its end", false, &format!("panicked: {text}"));
    }
    let closed = world.close();
    checks.check("removed the throwaway repositories", closed.is_ok(), &closed.map_or_else(|e| e.to_string(), |()| "removed".to_string()));
    let lines = checks.lines();
    block(block_verdict(&lines, panic.as_deref()), lines)
}

fn panic_text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| text.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic with no message".to_string())
}
