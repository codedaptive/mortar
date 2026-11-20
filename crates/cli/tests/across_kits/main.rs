//! The built mortar and mr binaries across kits, with folders, snapshot, join, land, gitio and oplog below cli and
//! real Git, against throwaway repositories: bisect_backup, after_publish, snapshot_overlap_disk_sweep and
//! undo_save_join_resolve. They share this target because their fixtures build on each other.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod bisect_backup;
mod after_publish;
mod snapshot_overlap_disk_sweep;
mod undo_save_join_resolve;
