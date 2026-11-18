//! `mr restore` (SPEC-cli 05, S-113, S-115): its lock and key, the folders on the refs it names and the moves
//! it makes, its refusals and reports, and its runner.

mod folder_on_ref;
mod folders_on_refs;
mod folders_to_move;
mod move_restored_folders;
mod ref_moved_on;
mod restore_error_report;
mod restore_key;
mod restore_lock;
mod restore_move_refusal_report;
mod restore_moves;
mod restore_refusal;
mod restore_report;
mod run_restore;
mod snapshot_value_to_check;

#[cfg(test)]
mod ref_moved_on_tests;
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
#[cfg(test)]
mod run_restore_tests;
#[cfg(test)]
mod snapshot_value_to_check_tests;

pub(crate) use folder_on_ref::FolderOnRef;
pub(crate) use folders_on_refs::folders_on_refs;
pub(crate) use folders_to_move::folders_to_move;
pub(crate) use move_restored_folders::move_restored_folders;
pub(crate) use ref_moved_on::ref_moved_on;
pub(crate) use restore_error_report::restore_error_report;
pub(crate) use restore_key::restore_key;
pub(crate) use restore_lock::restore_lock;
pub(crate) use restore_move_refusal_report::restore_move_refusal_report;
pub(crate) use restore_moves::restore_moves;
pub(crate) use restore_refusal::restore_refusal;
pub(crate) use restore_report::restore_report;
pub(crate) use run_restore::run_restore;
pub(crate) use snapshot_value_to_check::snapshot_value_to_check;
