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
mod closed_snap_refs_tests;
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
mod sweep_pick_tests;
#[cfg(test)]
mod sweep_refs_op_tests;

pub(crate) use closed_snap_refs::closed_snap_refs;
pub use default_build_dir::DEFAULT_BUILD_DIR;
pub(crate) use extra_ref::ExtraRef;
pub(crate) use extra_ref_verdict::extra_ref_verdict;
pub(crate) use folder_age::FolderAge;
pub(crate) use idle_folders::idle_folders;
pub(crate) use joined_leftovers::joined_leftovers;
pub(crate) use leftover::Leftover;
pub(crate) use leftover_branches::leftover_branches;
pub(crate) use stale_replayed_refs::stale_replayed_refs;
pub use sweep_build::sweep_build;
pub use sweep_build_outcome::SweepBuildOutcome;
pub(crate) use sweep_pick::sweep_pick;
pub use sweep_refs::sweep_refs;
pub(crate) use sweep_refs_op::sweep_refs_op;
