//! One step's throwaway repositories. Data only: World::new (world_new.rs) makes them and the shell step
//! (steps/shell.rs) checks what it made on every drive.

use std::ffi::OsString;
use std::path::PathBuf;

use super::Scratch;

/// Under `root`: `work`, the working repository (main and integration at `base`, remote `origin`); `origin.git`, the
/// bare remote holding `main`; `other`, a second clone of the remote, for "someone else". All are removed with the
/// scratch folder. `path` is every child's PATH: cargo's own build output folder, holding the `mr` and `mortar` it
/// built, then the drive's own PATH (world_bin_path).
pub struct World {
    pub root: PathBuf,
    pub work: PathBuf,
    pub origin: PathBuf,
    pub other: PathBuf,
    pub base: String,
    pub path: OsString,
    pub(super) scratch: Scratch,
}
