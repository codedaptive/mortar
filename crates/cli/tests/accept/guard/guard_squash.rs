//! Step 2's set-up after stage_new: a squash ready to land. Real repositories and processes: run_guard.rs runs it
//! on every drive.

use crate::report::Checks;
use crate::stage::{agent_mr, harness_mr, setup_step, stage_request, stage_save};
use crate::world::World;

/// In zone a: `mr folder request a r1 <root>/r1 --paths src/a`, a new file src/a/r1.txt saved with `mr save`,
/// `mr join` there, then `mr squash a`; one set-up line each. The squash's commit, or the first step that failed
/// and what it saw; the steps after a failed one are not run.
pub fn guard_squash(world: &World, checks: &mut Checks) -> Result<String, String> {
    let (r1, made) = stage_request(world, "a", "r1", &["src/a"]);
    setup_step(checks, "set-up: mr folder request a r1 <root>/r1 --paths src/a --actor accept", &made, "made")?;
    let saved = stage_save(world, &r1, "src/a/r1.txt", "r1\n", "r1: first");
    setup_step(checks, "set-up: write src/a/r1.txt in r1, mr save \"r1: first\"", &saved, "saved")?;
    let joined = agent_mr(world, &r1, &["join"]);
    setup_step(checks, "set-up: mr join in r1", &joined, "joined")?;
    let squash = harness_mr(world, &["squash", "a"]);
    let commit = squash.text("commit");
    let did = "set-up: mr squash a --actor accept";
    let saw = format!("{}; commit {commit}", squash.saw);
    if checks.check(did, squash.is(0, "squashed") && !commit.is_empty(), &saw) {
        return Ok(commit);
    }
    Err(format!("{did}: {saw}"))
}
