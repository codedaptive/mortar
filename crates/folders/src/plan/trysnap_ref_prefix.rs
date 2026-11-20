//! Where an attempt's snapshot ref lives: this namespace holds attempt snapshots. Folders writes none of
//! these refs; it restates the prefix so the sweeps can delete them once the request closes.
//! A constant; closed_snap_refs_tests exercises it.

/// The namespace of attempt snapshot refs: `refs/mortar/trysnap/<zone>/<request>/<k>`.
pub const TRYSNAP_REF_PREFIX: &str = "refs/mortar/trysnap/";
