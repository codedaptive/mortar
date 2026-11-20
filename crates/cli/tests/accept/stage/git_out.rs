//! Reads state with plain git. Real processes: step 1 (flow/run_flow.rs) reads refs, files and commits through it
//! on every drive.

use std::path::Path;

use crate::world::World;

/// The trimmed stdout of `git <args>` in `cwd`: empty when git found nothing or failed.
pub fn git_out(world: &World, cwd: &Path, args: &[&str]) -> String {
    world.git(cwd, args).stdout.trim().to_string()
}
