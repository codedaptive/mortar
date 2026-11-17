//! The commit step 2 tries to move guarded refs to. Real processes: run_guard.rs makes it on every drive.

use crate::stage::git_out;
use crate::world::World;

/// `git commit-tree <base>^{tree} -p <base> -m "someone else's commit"` in the working repository: a new commit on
/// the base that no ref names, so moving integration or main to it would be a fast-forward the guard alone stops.
/// "" when git made none.
pub fn other_commit(world: &World) -> String {
    let tree = format!("{}^{{tree}}", world.base);
    git_out(world, &world.work, &["commit-tree", &tree, "-p", &world.base, "-m", "someone else's commit"])
}
