//! `mortar restore` (SPEC-cli 05, S-113, S-115): its lock and key, the folders on the refs it names and the moves
//! it makes, its refusals and reports, and its runner.

mod folder_on_ref;
mod folders_on_refs;
mod folders_to_move;
mod move_restored_folders;
mod restore_error_report;
mod restore_key;
mod restore_lock;
mod restore_move_refusal_report;
mod restore_moves;
mod restore_refusal;
mod restore_report;
mod run_restore;

#[cfg(test)]
mod restore_error_report_tests;
#[cfg(test)]
mod restore_key_tests;
#[cfg(test)]
mod restore_lock_tests;
#[cfg(test)]
mod restore_move_refusal_report_tests;
#[cfg(test)]
mod restore_refusal_tests;
#[cfg(test)]
mod restore_report_tests;

pub use folder_on_ref::FolderOnRef;
pub use folders_on_refs::folders_on_refs;
pub use folders_to_move::folders_to_move;
pub use move_restored_folders::move_restored_folders;
pub use restore_error_report::restore_error_report;
pub use restore_key::restore_key;
pub use restore_lock::restore_lock;
pub use restore_move_refusal_report::restore_move_refusal_report;
pub use restore_moves::restore_moves;
pub use restore_refusal::restore_refusal;
pub use restore_report::restore_report;
pub use run_restore::run_restore;
