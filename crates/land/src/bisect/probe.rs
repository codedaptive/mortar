//! One probe of the bisect. A bare type (S-7); tests/bisect_zit exercises it, and its Display cases are Nagatha's
//! RITs (owed).

use std::path::PathBuf;

use gitio::Oid;

use super::ProbeOutcome;

/// The caller's command run once at one commit, in a throwaway worktree (07-bisect.md).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    /// The commit probed.
    pub commit: Oid,
    /// How the command ended there.
    pub outcome: ProbeOutcome,
    /// The detached throwaway worktree the command ran in; removed after the probe.
    pub folder: PathBuf,
}
