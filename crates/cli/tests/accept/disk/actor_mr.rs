//! Runs a harness verb as the drive's actor in the repository copy or a request folder. Real processes: the drive
//! (steps/disk.rs) runs `mr init` through it (disk_init) on every drive that passes the refusal and reads the plan
//! with the copy's HEAD on a branch, and once more to register the one-kit zone, after the all-packages zone was
//! measured, when that first init held and the plan has a kit. For the all-packages zone after the first init held,
//! and for the one-kit zone after the second held (measure_zone): `mr folder zone`; `mr folder request` only after
//! that zone's folder line held; `mr disk` and `mr snapshot` only for a zone whose two folders were made.

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
