//! 03's case 1: a join killed before its ref transaction. Real repositories, processes and a SIGKILL: run_kill.rs
//! runs it on every drive.

use super::{done_moves_of, kill_armed, kill_consistent, kill_log_check, read_log, CaseLines, KillState};
use crate::report::Checks;
use crate::stage::{agent_mr, git_out};
use crate::world::{Crash, World};

const ZONE: &str = "refs/heads/zone/a";

/// `mr join --json` in r1, a fast-forward of zone a whose first update-ref is the zone's, killed before it: the open
/// intent is a join-fast-forward; zone/a unmoved; `mr join` again exits 0 "joined" with zone/a at r1's tip; the log
/// shows the killed op closed failed (not-moved) and one done fast-forward; and since the kill one done op in all
/// moved zone/a. Then the consistency items, repeating `mr join` in r1. True when zone a moved, so case 2's rebase
/// can run.
pub fn kill_join_before(world: &World, checks: &mut Checks, state: &KillState) -> bool {
    let mut lines = CaseLines::new(0);
    let zone = || git_out(world, &world.work, &["rev-parse", "-q", "--verify", ZONE]);
    let before = zone();
    let armed = kill_armed(world, checks, &mut lines, (&state.r1, &["join", "--json"]), Crash::Before, "join-fast-forward");
    let joined = match armed {
        None => {
            lines.not_run(checks, "the kill did not stop a join at its ref transaction", false);
            false
        }
        Some((killed, _)) => {
            let after = zone();
            lines.check(checks, after == before, &format!("zone/a {before} before the kill, {after} after"));
            let join = agent_mr(world, &state.r1, &["join"]);
            let (moved, tip) = (zone(), git_out(world, &world.work, &["rev-parse", "-q", "--verify", "req/a/r1"]));
            let saw = format!("{}; zone/a {moved}, req/a/r1 {tip}", join.saw);
            let joined = lines.check(checks, join.is(0, "joined") && moved == tip, &saw);
            kill_log_check(world, checks, &mut lines, &killed, ("failed", "not-moved"), ZONE);
            let count = read_log(world).map(|log| done_moves_of(&log, ZONE, &killed.op_id).len());
            let saw = format!("done ops moving zone/a since the kill: {count:?}; zone/a {before} -> {moved}");
            lines.check(checks, count == Ok(1) && moved == tip && moved != before, &saw);
            joined
        }
    };
    kill_consistent(world, checks, &mut lines, &state.folders, &|| agent_mr(world, &state.r1, &["join"]));
    joined
}
