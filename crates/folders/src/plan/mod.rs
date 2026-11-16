//! A folder's plan as data: branch, attempt and snapshot ref names, attempt numbers, attempt folder paths, sparse
//! paths, the free-disk floor, the branch op.

mod attempt_branch;
mod attempt_number;
mod attempt_path;
mod below_floor;
mod below_floor_display;
mod branch_op;
mod check_name;
mod cone_folders;
mod default_floor_bytes;
mod df_free_disk;
mod floor_check;
mod free_disk;
mod plan_error;
mod plan_error_display;
mod read_df_available;
mod request_branch;
mod snap_ref;
mod snap_ref_prefix;
mod sparse_paths;
mod sweep_estimate;
mod trysnap_ref_prefix;
mod zone_branch;

#[cfg(test)]
mod attempt_branch_tests;
#[cfg(test)]
mod attempt_number_tests;
#[cfg(test)]
mod attempt_path_tests;
#[cfg(test)]
mod below_floor_display_tests;
#[cfg(test)]
mod branch_op_tests;
#[cfg(test)]
mod check_name_tests;
#[cfg(test)]
mod cone_folders_tests;
#[cfg(test)]
mod floor_check_tests;
#[cfg(test)]
mod plan_error_display_tests;
#[cfg(test)]
mod read_df_available_tests;
#[cfg(test)]
mod request_branch_tests;
#[cfg(test)]
mod snap_ref_tests;
#[cfg(test)]
mod sparse_paths_tests;
#[cfg(test)]
mod zone_branch_tests;

pub use attempt_branch::attempt_branch;
pub(crate) use attempt_number::attempt_number;
pub use attempt_path::attempt_path;
pub use below_floor::BelowFloor;
pub use branch_op::branch_op;
pub(crate) use check_name::check_name;
pub(crate) use cone_folders::cone_folders;
pub use default_floor_bytes::DEFAULT_FLOOR_BYTES;
pub use df_free_disk::DfFreeDisk;
pub use floor_check::floor_check;
pub use free_disk::FreeDisk;
pub use plan_error::PlanError;
pub use read_df_available::read_df_available;
pub use request_branch::request_branch;
pub use snap_ref::snap_ref;
pub use snap_ref_prefix::SNAP_REF_PREFIX;
pub use sparse_paths::sparse_paths;
pub use sweep_estimate::SweepEstimate;
pub use trysnap_ref_prefix::TRYSNAP_REF_PREFIX;
pub use zone_branch::zone_branch;
