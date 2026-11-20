//! Step 1's set-up as the steps carry it. Data only: stage_new makes it.

use std::path::PathBuf;

/// The standing folders of zones a and b.
#[derive(Debug, Default)]
pub struct Stage {
    pub zone_a: PathBuf,
    pub zone_b: PathBuf,
}
