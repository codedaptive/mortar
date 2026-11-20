//! Runs one Done step into its report block. Real repositories and processes: the drive (main.rs, `done_list`) runs
//! every step through it.

use std::any::Any;
use std::ffi::OsString;
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::{decide_step, step_verdict, Decision, StepSpec};
use crate::report::{block_verdict, Block, Checks, Line, Verdict};
use crate::world::World;

/// A step that does not run is a NOT RUN block with no lines. One that runs gets a fresh world; a world that
/// cannot be made fails the step. The body runs with its panic caught, so one step's panic is a FAIL line in its
/// own block and the next steps still run. The body is judged by block_verdict on its own lines alone, so a body
/// that recorded no check fails. Then the world is removed and that is recorded as one more check line: a failed
/// removal turns the verdict to Fail, and a successful one never changes it (step_verdict).
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
    let mut lines = checks.lines();
    let body_verdict = block_verdict(&lines, panic.as_deref());
    let closed = world.close();
    let cleanup_ok = closed.is_ok();
    lines.push(Line::Check {
        ok: cleanup_ok,
        did: "removed the throwaway repositories".to_string(),
        saw: closed.map_or_else(|error| error.to_string(), |()| "removed".to_string()),
    });
    block(step_verdict(body_verdict, cleanup_ok), lines)
}

fn panic_text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| text.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic with no message".to_string())
}
