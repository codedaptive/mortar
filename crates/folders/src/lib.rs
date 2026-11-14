//! folders: see the Mortar mission (Layout).

pub mod make;
pub mod moves;
pub mod need;
pub mod plan;
pub mod record;

pub use make::{make_request_folder, make_zone_folder, FolderError, FolderSetup};
pub use moves::{move_folder, move_op};
pub use need::{gaps, need};
pub use plan::{DfFreeDisk, FreeDisk, SweepEstimate, DEFAULT_FLOOR_BYTES};
pub use record::{FolderKind, FolderRecord};
