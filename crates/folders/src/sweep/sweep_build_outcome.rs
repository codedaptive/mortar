//! What sweep_build cleared.
//! A bare type; the integration group covers it through sweep_build's cases.

use std::path::PathBuf;

/// The build dirs sweep_build deleted, in the order it found their folders, and the bytes they held (apparent
/// size, as measure_folder measured them before the sweep began).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepBuildOutcome {
    pub removed: Vec<PathBuf>,
    pub freed_bytes: u64,
}
