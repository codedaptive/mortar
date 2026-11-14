//! RefusedFile: a new file the size cap left out of a snapshot. A declaration only; cap_refusals_tests and
//! snapshot_from_apply_tests exercise it.

use std::path::PathBuf;

/// A new file over the cap, left out of the snapshot: its path in the folder, its size and the cap, both in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusedFile {
    pub path: PathBuf,
    pub size: u64,
    pub cap: u64,
}
