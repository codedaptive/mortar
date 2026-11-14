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

pub use attempts::{drop_attempts, drop_op, keep, keep_op, make_attempts, remake_orphans};
pub use disk::{disk, DiskReport, FolderDisk};
pub use make::{make_request_folder, make_zone_folder, FolderError, FolderSetup};
pub use moves::{move_folder, move_op};
pub use need::{gaps, need};
pub use plan::{DfFreeDisk, FreeDisk, SweepEstimate, DEFAULT_FLOOR_BYTES};
pub use record::{FolderKind, FolderRecord};
pub use remove::{remove, remove_op};
pub use sweep::{sweep_build, sweep_refs, SweepBuildOutcome, DEFAULT_BUILD_DIR};
