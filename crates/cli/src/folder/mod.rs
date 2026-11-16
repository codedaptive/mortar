//! `mortar folder` (SPEC-cli 05): its words, the paths a folder is cut on, the decisions its sub-verbs make from
//! data, its refusals and its reports, and the runners of its sub-verbs.

mod attempt_saved;
mod attempts_dir;
mod attempts_made_report;
mod attempts_saved;
mod dropped_report;
mod folder_command;
mod folder_error_report;
mod folder_error_why;
mod folder_made_report;
mod folder_refusal;
mod folder_removed_report;
mod folder_setup_for;
mod kept_report;
mod parse_folder_args;
mod pending_keep_move;
mod request_folder_paths;
mod request_folder_refusal;
mod run_folder;
mod run_folder_attempts;
mod run_folder_drop;
mod run_folder_keep;
mod run_folder_remove;
mod run_folder_request;
mod run_folder_zone;
mod zone_folder_paths;

#[cfg(test)]
mod attempt_saved_tests;
#[cfg(test)]
mod attempts_dir_tests;
#[cfg(test)]
mod attempts_made_report_tests;
#[cfg(test)]
mod dropped_report_tests;
#[cfg(test)]
mod folder_error_why_tests;
#[cfg(test)]
mod folder_made_report_tests;
#[cfg(test)]
mod folder_refusal_tests;
#[cfg(test)]
mod folder_removed_report_tests;
#[cfg(test)]
mod folder_setup_for_tests;
#[cfg(test)]
mod kept_report_tests;
#[cfg(test)]
mod parse_folder_args_tests;
#[cfg(test)]
mod pending_keep_move_tests;
#[cfg(test)]
mod request_folder_paths_tests;
#[cfg(test)]
mod request_folder_refusal_tests;
#[cfg(test)]
mod zone_folder_paths_tests;

pub use attempt_saved::attempt_saved;
pub use attempts_dir::attempts_dir;
pub use attempts_made_report::attempts_made_report;
pub use attempts_saved::attempts_saved;
pub use dropped_report::dropped_report;
pub use folder_command::FolderCommand;
pub use folder_error_report::folder_error_report;
pub use folder_error_why::folder_error_why;
pub use folder_made_report::folder_made_report;
pub use folder_refusal::folder_refusal;
pub use folder_removed_report::folder_removed_report;
pub use folder_setup_for::folder_setup_for;
pub use kept_report::kept_report;
pub use parse_folder_args::parse_folder_args;
pub use pending_keep_move::pending_keep_move;
pub use request_folder_paths::request_folder_paths;
pub use request_folder_refusal::request_folder_refusal;
pub use run_folder::run_folder;
pub use run_folder_attempts::run_folder_attempts;
pub use run_folder_drop::run_folder_drop;
pub use run_folder_keep::run_folder_keep;
pub use run_folder_remove::run_folder_remove;
pub use run_folder_request::run_folder_request;
pub use run_folder_zone::run_folder_zone;
pub use zone_folder_paths::zone_folder_paths;
