//! A repository's git common dir, read with plain git. Real repositories and processes: the drive (steps/disk.rs)
//! runs it through run_disk on every drive whose ACCEPT_MOOT_COPY canonicalized, for the copy; then, once the copy's
//! git common dir was read, for each registered repository that exists, in the order registered_repos reads them,
//! which stops at the first failure (ACCEPT_TEAMS, the manifests folder, a manifest or a repository): not every
//! registered repository is necessarily read.

use std::path::{Path, PathBuf};

use crate::world::World;

/// `git rev-parse --git-common-dir` run in `path`, joined to `path` when git answers a relative path, then
/// canonicalized. Err when git fails (naming `path` and how git ended) or the answer cannot be canonicalized, so a
/// caller deciding a refusal fails closed.
pub fn git_common_dir(world: &World, path: &Path) -> Result<PathBuf, String> {
    let ran = world.git(path, &["rev-parse", "--git-common-dir"]);
    if ran.code != Some(0) {
        return Err(format!("git rev-parse --git-common-dir in {} failed: {}", path.display(), ran.seen()));
    }
    let common = path.join(ran.stdout.trim());
    common
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize the git common dir {}: {error}", common.display()))
}
