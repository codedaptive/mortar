//! Step 5's refusal decision on the moot copy. Real files, repositories and processes: the drive (steps/disk.rs)
//! runs it through run_disk, before anything else, on every drive with ACCEPT_MOOT_COPY set and not empty whose
//! throwaway world was made.

use std::path::{Path, PathBuf};

use super::{copy_refusal, git_common_dir, registered_repos};
use crate::world::World;

/// ACCEPT_MOOT_COPY canonicalized, with the number of registered repositories read, when copy_refusal finds it is
/// neither a registered team repository nor a worktree of one. Err with the reason otherwise, and also when the
/// variable is unset, the copy or its git common dir cannot be read, or registered_repos fails: it fails closed.
/// It reads the copy, its git common dir and the team manifests, and writes nothing.
pub fn writable_copy(world: &World) -> Result<(PathBuf, usize), String> {
    let given = std::env::var_os("ACCEPT_MOOT_COPY").ok_or("ACCEPT_MOOT_COPY is not set")?;
    let given = Path::new(&given);
    let copy = given
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize ACCEPT_MOOT_COPY {}: {error}", given.display()))?;
    let common = git_common_dir(world, &copy)?;
    let registered = registered_repos(world)?;
    match copy_refusal(&copy, &common, &registered) {
        Some(why) => Err(why),
        None => Ok((copy, registered.len())),
    }
}
