//! The team repositories codeteam's manifests register. Real files, repositories and processes: the drive
//! (steps/disk.rs) runs it through run_disk on every drive where the copy and its git common dir were read.

use std::path::{Path, PathBuf};

use super::{git_common_dir, manifest_repos, teams_folder, Registered};
use crate::world::World;

/// Every `repo = "<path>"` line (manifest_repos) of every `*.toml` in the manifests folder (ACCEPT_TEAMS, set and
/// not empty), the manifests in sorted order. A path that exists is canonicalized, with its git common dir; one
/// that does not is kept as given with no common dir. Err, naming what failed, when ACCEPT_TEAMS is unset or empty
/// (teams_folder), the folder or a manifest cannot be read, or an existing path or its common dir cannot be read:
/// the refusal fails closed. No repo line anywhere is Ok and empty (copy_refusal then refuses).
pub fn registered_repos(world: &World) -> Result<Vec<Registered>, String> {
    let folder = teams_folder(std::env::var_os("ACCEPT_TEAMS"))?;
    let mut registered = Vec::new();
    for manifest in manifests(&folder)? {
        let text = std::fs::read_to_string(&manifest)
            .map_err(|error| format!("cannot read the team manifest {}: {error}", manifest.display()))?;
        for repo in manifest_repos(&text) {
            registered.push(registered_repo(world, Path::new(&repo))?);
        }
    }
    Ok(registered)
}

/// The `*.toml` files directly in `folder`, sorted.
fn manifests(folder: &Path) -> Result<Vec<PathBuf>, String> {
    let unread = |error: std::io::Error| format!("cannot read the team manifests folder {}: {error}", folder.display());
    let mut paths = std::fs::read_dir(folder)
        .map_err(unread)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(unread)?;
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "toml"));
    paths.sort();
    Ok(paths)
}

fn registered_repo(world: &World, path: &Path) -> Result<Registered, String> {
    let exists =
        path.try_exists().map_err(|error| format!("cannot tell whether {} exists: {error}", path.display()))?;
    if !exists {
        return Ok(Registered { path: path.to_path_buf(), common: None });
    }
    let path = path.canonicalize().map_err(|error| format!("cannot canonicalize {}: {error}", path.display()))?;
    let common = git_common_dir(world, &path)?;
    Ok(Registered { path, common: Some(common) })
}
