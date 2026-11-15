//! Sweeps: idle folders' build dirs, and refs: closed requests' snapshot refs (attempts' included), replay-record
//! refs of zones the caller no longer lists, joined leftover request branches with their snapshot refs, and refs the
//! caller names.

mod closed_snap_refs;
mod default_build_dir;
mod extra_ref;
mod extra_ref_verdict;
mod folder_age;
mod idle_folders;
mod joined_leftovers;
mod leftover;
mod leftover_branches;
mod stale_replayed_refs;
mod sweep_build;
mod sweep_build_outcome;
mod sweep_pick;
mod sweep_refs;
mod sweep_refs_op;

#[cfg(test)]
mod extra_ref_verdict_tests;
#[cfg(test)]
mod idle_folders_tests;
#[cfg(test)]
mod joined_leftovers_tests;
#[cfg(test)]
mod leftover_branches_tests;
#[cfg(test)]
mod stale_replayed_refs_tests;
#[cfg(test)]
mod sweep_refs_op_tests;

pub use closed_snap_refs::closed_snap_refs;
pub use default_build_dir::DEFAULT_BUILD_DIR;
pub use extra_ref::ExtraRef;
pub use extra_ref_verdict::extra_ref_verdict;
pub use folder_age::FolderAge;
pub use idle_folders::idle_folders;
pub use joined_leftovers::joined_leftovers;
pub use leftover::Leftover;
pub use leftover_branches::leftover_branches;
pub use stale_replayed_refs::stale_replayed_refs;
pub use sweep_build::sweep_build;
pub use sweep_build_outcome::SweepBuildOutcome;
pub use sweep_pick::sweep_pick;
pub use sweep_refs::sweep_refs;
pub use sweep_refs_op::sweep_refs_op;
