//! Step 3's set-up as the cases carry it. Data only: kill_setup makes it.

use std::path::PathBuf;

/// The request folders r1 and r2 (zone a, each with a saved edit, neither joined) and r3 (zone b, saved and
/// joined), and every folder the step made, zone folders first, for the consistency check.
#[derive(Debug, Default)]
pub struct KillState {
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub r3: PathBuf,
    pub folders: Vec<PathBuf>,
}
