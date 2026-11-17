//! Reads the bare remote with plain git. Real processes: steps 1-3 read origin's refs through it on every drive.

use crate::stage::git_out;
use crate::world::World;

/// The trimmed stdout of `git --git-dir <origin.git> <args>`: empty when git found nothing or failed.
pub fn remote_git(world: &World, args: &[&str]) -> String {
    let origin = world.origin.to_string_lossy();
    let mut all = vec!["--git-dir", origin.as_ref()];
    all.extend_from_slice(args);
    git_out(world, &world.root, &all)
}
