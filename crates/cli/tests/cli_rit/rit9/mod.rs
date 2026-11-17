//! WO-cli-rit-9: CLI run_snapshot/run_overlap/run_disk/run_sweep and read_named_zones joined to real
//! Snapshot/Join/Folders/Oplog/Gitio. Covers 5b's runner inventories and S-141, not a speed or acceptance drive.
//! Reuses owned isolated repositories/worktrees/remotes; support::Fixture removes the whole disposable root.
mod fixture;
mod snapshots;
mod overlap;
mod disk;
mod sweep;
mod guards;
