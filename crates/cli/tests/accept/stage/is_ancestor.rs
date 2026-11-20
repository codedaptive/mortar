//! Whether one commit holds another, read with plain git. Real processes: steps 1-3 read it on every drive.

use crate::world::World;

/// True when `git merge-base --is-ancestor <ancestor> <descendant>` in the working repository exits 0.
pub fn is_ancestor(world: &World, ancestor: &str, descendant: &str) -> bool {
    world.git(&world.work, &["merge-base", "--is-ancestor", ancestor, descendant]).code == Some(0)
}
