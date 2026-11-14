//! Making a folder: a zone's standing folder or a request's, under the folders lock, recorded and filled.

mod attach_head;
mod attach_head_request;
mod branch_holder;
mod branch_step;
mod branch_step_after;
mod check_before_make;
mod existing_folder;
mod fill_folder;
mod folder_error;
mod folder_error_display;
mod folder_error_source;
mod folder_setup;
mod make_branch;
mod make_request_folder;
mod make_zone_folder;
mod read_folder_record;
mod refill_cone;
mod refill_folder;
mod repeat;
mod repeat_decision;
mod skip_refill;
mod with_folders_lock;
mod write_folder_record;
mod zone_start;
mod zone_start_for;

#[cfg(test)]
mod attach_head_request_tests;
#[cfg(test)]
mod branch_holder_tests;
#[cfg(test)]
mod branch_step_after_tests;
#[cfg(test)]
mod folder_error_display_tests;
#[cfg(test)]
mod folder_error_source_tests;
#[cfg(test)]
mod refill_cone_tests;
#[cfg(test)]
mod repeat_decision_tests;
#[cfg(test)]
mod skip_refill_tests;
#[cfg(test)]
mod zone_start_for_tests;

pub use branch_holder::branch_holder;
pub(crate) use branch_step::BranchStep;
pub use existing_folder::existing_folder;
pub use folder_error::FolderError;
pub use folder_setup::FolderSetup;
pub use make_request_folder::make_request_folder;
pub use make_zone_folder::make_zone_folder;
pub use read_folder_record::read_folder_record;
pub use repeat::Repeat;
pub use repeat_decision::repeat_decision;
pub(crate) use with_folders_lock::with_folders_lock;
pub use write_folder_record::write_folder_record;
pub(crate) use zone_start::ZoneStart;
