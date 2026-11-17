//! Step 1's check 4: a zone moves under a request and the join rebases it. Real repositories and processes:
//! run_flow.rs runs it on every drive.

use super::{not_run, FlowState};
use crate::report::Checks;
use crate::stage::{agent_mr, git_out, is_ancestor, stage_save};
use crate::world::World;

const R2_TEXT: &str = "r2 line 1\nr2 line 2\nr2 line 3\nr2 line 4\nr2 line 5\nr2 line 6\nr2 line 7\nr2 line 8\n";

/// r2 saves a new file of src/a while zone a already holds r1 (check 3); `mr join` in r2 must answer rebased with
/// next `mr join` and the line that the unit tests must run again, and r2's folder must stand at the new tip (its
/// HEAD at req/a/r2 and at the reply's `to`, r1's file in it). False, with the later checks written as not run,
/// when the join did not rebase.
pub fn flow_rebase(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let save = stage_save(world, &state.r2, "src/a/r2.txt", R2_TEXT, "r2: second");
    let zone_has_r1 = is_ancestor(world, "req/a/r1", "zone/a");
    let join = agent_mr(world, &state.r2, &["join"]);
    let head = git_out(world, &state.r2, &["rev-parse", "HEAD"]);
    let tip = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "req/a/r2"]);
    let has_r1 = state.r2.join("src/a/r1.txt").is_file();
    let again = join.lines.iter().any(|line| line == "the unit tests must run again");
    let moved = !tip.is_empty() && head == tip && join.text("to") == tip && has_r1;
    let rebased = join.is(0, "rebased");
    let ok = save.is(0, "saved") && zone_has_r1 && rebased && join.next == "mr join" && again && moved;
    let saw = format!(
        "save: {}; zone/a holds r1: {zone_has_r1}; join: {}; next: {}; r2 HEAD {head}, req/a/r2 {tip}, to {}; \
         src/a/r1.txt in r2: {has_r1}; \"the unit tests must run again\" said: {again}",
        save.saw,
        join.saw,
        join.next,
        join.text("to")
    );
    checks.check("4. write src/a/r2.txt in r2, mr save \"r2: second\", mr join in r2", ok, &saw);
    if !rebased {
        not_run(checks, 5, "check 4 failed");
    }
    rebased
}
