//! Step 3, nothing lost, nothing twice. Real repositories, processes and SIGKILLs: the drive
//! (steps/kill.rs) runs it as step 3's body on every drive.

use super::{kill_join_after, kill_join_before, kill_land, kill_not_run, kill_publish, kill_setup, squash_ready};
use crate::report::Checks;
use crate::stage::harness_mr;
use crate::world::{Crash, World};

/// The set-up (kill_setup), then the cases in order, each followed by the consistency items: 1 a join killed
/// before (r1), 2 a join killed after (r2, rebased by case 1's move); 3 a land killed before (zone a's squash), then,
/// once that landing is published, one killed after (zone b's squash); 4 the publish of that second landing, killed
/// between its push and its local move. A case that needs what an earlier one failed to leave is written as not
/// run, with every case after it, so the block always lists every check.
pub fn run_kill(world: &World, checks: &mut Checks) {
    let state = match kill_setup(world, checks) {
        Ok(state) => state,
        Err(why) => return kill_not_run(checks, 0, &why),
    };
    if !kill_join_before(world, checks, &state) {
        return kill_not_run(checks, 1, "case 1's join did not move zone a, so nothing can rebase r2");
    }
    kill_join_after(world, checks, &state);
    let first = match squash_ready(world, checks, "a") {
        Ok(commit) => commit,
        Err(why) => return kill_not_run(checks, 2, &why),
    };
    if !kill_land(world, checks, &state, 2, &first, Crash::Before) {
        return kill_not_run(checks, 3, "case 3's first landing did not land");
    }
    let publish = harness_mr(world, &["publish"]);
    if !checks.check("set-up: mr publish --actor accept (the first landing)", publish.is(0, "published"), &publish.saw) {
        return kill_not_run(checks, 3, "the first landing was not published, so integration is not free");
    }
    let second = match squash_ready(world, checks, "b") {
        Ok(commit) => commit,
        Err(why) => return kill_not_run(checks, 3, &why),
    };
    if !kill_land(world, checks, &state, 3, &second, Crash::After) {
        return kill_not_run(checks, 4, "case 3's second landing did not land, so there is nothing to publish");
    }
    kill_publish(world, checks, &state, &second);
}
