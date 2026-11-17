//! Step 3's set-up. Real repositories and processes: run_kill.rs runs it on every drive.

use super::KillState;
use crate::report::Checks;
use crate::stage::{agent_mr, setup_step, stage_new, stage_request, stage_save};
use crate::world::World;

/// stage_new, then request folders r1 and r2 in zone a and r3 in zone b, a new file saved with `mr save` in each
/// (so a join's own op is its first ref transaction: the save leaves nothing for join to save or snapshot), and
/// `mr join` in r3; one set-up line each. r1 and r2 both start at zone a's base, so once r1 joins, zone a has moved
/// under r2. An error naming the first step that failed; the steps after it are not run.
pub fn kill_setup(world: &World, checks: &mut Checks) -> Result<KillState, String> {
    let stage = stage_new(world, checks).ok_or("the set-up (init, guard install, zone folders) failed")?;
    let mut state = KillState { folders: vec![stage.zone_a, stage.zone_b], ..KillState::default() };
    for (zone, request, path) in [("a", "r1", "src/a"), ("a", "r2", "src/a"), ("b", "r3", "src/b")] {
        let (folder, made) = stage_request(world, zone, request, &[path]);
        let did = format!("set-up: mr folder request {zone} {request} <root>/{request} --paths {path} --actor accept");
        setup_step(checks, &did, &made, "made")?;
        let file = format!("{path}/{request}.txt");
        let saved = stage_save(world, &folder, &file, &format!("{request}\n"), &format!("{request}: kill case"));
        setup_step(checks, &format!("set-up: write {file} in {request}, mr save"), &saved, "saved")?;
        state.folders.push(folder);
    }
    (state.r1, state.r2, state.r3) = (world.root.join("r1"), world.root.join("r2"), world.root.join("r3"));
    let joined = agent_mr(world, &state.r3, &["join"]);
    setup_step(checks, "set-up: mr join in r3", &joined, "joined")?;
    Ok(state)
}
