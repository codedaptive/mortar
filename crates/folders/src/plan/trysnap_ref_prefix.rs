//! Where an attempt's snapshot ref lives: S-69 names this namespace for attempt snapshots. Folders writes none of
//! these refs; it restates the prefix so the sweeps can delete them once the request closes.
//! A constant; tests/folders/attempt_plans.rs (closed_snap_refs) exercises it.

/// The namespace of attempt snapshot refs: `refs/mortar/trysnap/<zone>/<request>/<k>`.
pub const TRYSNAP_REF_PREFIX: &str = "refs/mortar/trysnap/";
