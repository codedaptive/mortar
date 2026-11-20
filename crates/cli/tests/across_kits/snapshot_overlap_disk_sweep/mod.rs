//! CLI run_snapshot/run_overlap/run_disk/run_sweep and read_named_zones joined to real
//! Snapshot/Join/Folders/Oplog/Gitio. Covers the runners' inventories and the sweep policy, not a speed or
//! acceptance drive.
//! Reuses owned isolated repositories/worktrees/remotes; support::Fixture removes the whole disposable root.
pub(super) mod fixture;
mod snapshots;
mod overlap;
mod disk;
mod sweep;
mod guards;
