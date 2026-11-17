//! Runs a harness verb as the drive's actor in the moot copy or the request folder. Real processes: the drive
//! (steps/disk.rs) runs `mr init` through it on every drive that passes the refusal and reads the plan with the
//! copy's HEAD on a branch; `mr folder zone` only after the init line held, `mr folder request` only after the zone
//! line held; `mr disk` and `mr snapshot` only on drives whose two folders were made.

use std::path::Path;

use crate::stage::{read_reply, Reply};
use crate::world::World;

/// `mr <args> --actor accept --json` in `cwd`, read by read_reply. Unlike stage's harness_mr, the folder is the
/// caller's (the copy or one of its folders), never the world's working repository.
pub fn actor_mr(world: &World, cwd: &Path, args: &[&str]) -> Reply {
    let mut all = args.to_vec();
    all.extend(["--actor", "accept", "--json"]);
    read_reply(&world.mortar(cwd, &all))
}
