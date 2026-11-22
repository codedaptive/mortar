//! folders: see the Mortar mission (Layout).

pub mod attempts;
pub mod disk;
pub mod make;
pub mod moves;
pub mod need;
pub mod plan;
pub mod record;
pub mod remove;
pub mod sweep;

#[cfg(test)]
mod test_ref_pairs;

pub use attempts::{drop_attempts, keep, make_attempts, remake_orphans};
pub use disk::{disk, folder_unsaved, DiskReport, FolderDisk, UnsavedCounts};
pub use make::{make_request_folder, make_zone_folder, set_zone_paths, FolderError, FolderSetup};
pub use moves::move_folder;
pub use need::{gaps, need};
pub use plan::{DfFreeDisk, FreeDisk, SweepEstimate, DEFAULT_FLOOR_BYTES};
pub use record::{FolderKind, FolderRecord};
pub use remove::remove;
pub use sweep::{sweep_build, sweep_refs, SweepBuildOutcome, DEFAULT_BUILD_DIR};
