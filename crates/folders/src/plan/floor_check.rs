//! The free-disk floor: may a folder be made with this much free disk?

use super::{BelowFloor, SweepEstimate};

/// Ok when `free` >= `floor`; else BelowFloor carrying the three values for the refusal's text.
pub fn floor_check(free: u64, floor: u64, sweep: &SweepEstimate) -> Result<(), BelowFloor> {
    if free >= floor {
        return Ok(());
    }
    Err(BelowFloor {
        free,
        floor,
        sweep: sweep.clone(),
    })
}
