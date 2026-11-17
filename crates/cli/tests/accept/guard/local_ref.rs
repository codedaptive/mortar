//! Reads a ref of the working repository with plain git. Real processes: step 2 (run_guard.rs) reads the refs it
//! checks through it on every drive.

use crate::stage::git_out;
use crate::world::World;

/// The commit `refname` names in the working repository (its linked worktrees share it); "" when it is absent.
pub fn local_ref(world: &World, refname: &str) -> String {
    git_out(world, &world.work, &["rev-parse", "-q", "--verify", refname])
}
