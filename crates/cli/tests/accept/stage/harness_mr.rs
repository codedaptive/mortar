//! Runs a harness verb as the drive's actor. Real processes: step 1 (flow/run_flow.rs) runs every harness verb
//! through it on every drive.

use super::{read_reply, Reply};
use crate::world::World;

/// `mr <args> --actor accept --json` in the working repository, read by read_reply.
pub fn harness_mr(world: &World, args: &[&str]) -> Reply {
    let mut all = args.to_vec();
    all.extend(["--actor", "accept", "--json"]);
    read_reply(&world.mortar(&world.work, &all))
}
