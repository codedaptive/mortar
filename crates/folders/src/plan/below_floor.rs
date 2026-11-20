//! The refusal to make a folder when free disk is under the floor.
//! A bare type; floor_check_tests and below_floor_display_tests exercise it.

use super::SweepEstimate;

/// `free` and `floor` in bytes, and what a sweep would free, so the refusal can say what to sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BelowFloor {
    pub free: u64,
    pub floor: u64,
    pub sweep: SweepEstimate,
}
