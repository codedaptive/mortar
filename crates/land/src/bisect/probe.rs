//! One probe of the bisect. A bare type; tests/bisect_zit exercises it, and its Display cases are repo integration
//! tests (RITs): tests/land_rit/owed_displays.rs covers them.

use std::path::PathBuf;

use gitio::Oid;

use super::ProbeOutcome;

/// The caller's command run once at one commit, in a throwaway worktree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    /// The commit probed.
    pub commit: Oid,
    /// How the command ended there.
    pub outcome: ProbeOutcome,
    /// The detached throwaway worktree the command ran in; removed after the probe.
    pub folder: PathBuf,
}
