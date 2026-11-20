//! Where a request's snapshot ref lives. The snapshot crate owns these refs; folders cannot depend on
//! snapshot, a sibling crate, so the prefix is restated here for remove and the sweeps.

/// The namespace of snapshot refs: `refs/mortar/snap/<zone>/<request>`.
pub const SNAP_REF_PREFIX: &str = "refs/mortar/snap/";
