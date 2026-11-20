//! Runs an agent verb in a request folder. Real processes: step 1 (flow/run_flow.rs) runs join and resolve
//! through it on every drive.

use std::path::Path;

use super::{read_reply, Reply};
use crate::world::World;

/// `mr <args> --json` in `folder`, with no actor (the agent verbs take theirs from the folder), read by read_reply.
pub fn agent_mr(world: &World, folder: &Path, args: &[&str]) -> Reply {
    let mut all = args.to_vec();
    all.push("--json");
    read_reply(&world.mortar(folder, &all))
}
