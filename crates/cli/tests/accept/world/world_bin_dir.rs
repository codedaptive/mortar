//! The world's bin folder: the built binary under both its names, put first on every child's PATH (World::new),
//! so the drive runs `mr` and the guard hooks Mortar installs run `mortar`.

use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use super::World;

/// Makes `<root>/bin` holding two symlinks, `mr` to World::mr_bin and `mortar` to World::mortar_bin, and returns
/// its path. Fails if `<root>/bin` already exists.
pub fn world_bin_dir(root: &Path) -> io::Result<PathBuf> {
    let bin = root.join("bin");
    std::fs::create_dir(&bin)?;
    symlink(World::mr_bin(), bin.join("mr"))?;
    symlink(World::mortar_bin(), bin.join("mortar"))?;
    Ok(bin)
}
