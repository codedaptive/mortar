//! The working repository's git common dir, where Mortar keeps its op log, locks and landing state. Real processes:
//! run_kill.rs reads the locks and the landing state under it on every drive.

use std::path::PathBuf;

use crate::stage::git_out;
use crate::world::World;

/// `git rev-parse --git-common-dir` in the working repository, made absolute against it when git answers a
/// relative path.
pub fn common_dir(world: &World) -> PathBuf {
    world.work.join(git_out(world, &world.work, &["rev-parse", "--git-common-dir"]))
}
