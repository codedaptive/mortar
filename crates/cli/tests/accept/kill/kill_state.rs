//! Step 3's set-up as the cases carry it. Data only: kill_setup makes it.

use std::path::PathBuf;

/// The request folders r1 and r2 (zone a, each with a saved edit, neither joined) and r3 (zone b, saved and
/// joined), and `folders`, those request folders, for the consistency check (no zone folder: one moves only after
/// publish, S-115 (1), S-140, S-150).
#[derive(Debug, Default)]
pub struct KillState {
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub r3: PathBuf,
    pub folders: Vec<PathBuf>,
}
