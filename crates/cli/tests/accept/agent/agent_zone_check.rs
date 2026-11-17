//! Step 4's check 1: the agent's work is on zone a. Real processes: run_agent.rs runs it on every drive with
//! ACCEPT_AGENT set.

use super::AGENT_CHECKS;
use crate::report::Checks;
use crate::stage::{git_out, holds_conflict_markers};
use crate::world::World;

/// The lines AGENT_BRIEF asks for and the line the drive's conflicting change writes (write_agent_bin).
const AGENT_LINE: &str = "a: agent";
const DRIVE_LINE: &str = "a: drive";
const ADDED: &str = "agent was here";

/// Read with plain git in the working repository: zone/a's src/a/a.txt holds a line `a: agent` and a line
/// `a: drive` (both sides' intent) and no conflict markers, and zone/a's src/a/agent.txt is `agent was here`.
pub fn agent_zone_check(world: &World, checks: &mut Checks) {
    let shared = git_out(world, &world.work, &["show", "zone/a:src/a/a.txt"]);
    let added = git_out(world, &world.work, &["show", "zone/a:src/a/agent.txt"]);
    let lines: Vec<&str> = shared.lines().collect();
    let both = lines.contains(&AGENT_LINE) && lines.contains(&DRIVE_LINE) && !holds_conflict_markers(&shared);
    let saw = format!("zone/a's src/a/a.txt {shared:?}; zone/a's src/a/agent.txt {added:?}");
    checks.check(AGENT_CHECKS[1], both && added == ADDED, &saw);
}
