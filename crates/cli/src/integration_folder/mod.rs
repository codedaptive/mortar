//! The integration folder: the worktree on the integration branch, found from git, moved after a verb moves that
//! branch, and its report line and refusal in the cli's own words.

mod folder_move_refusal_report;
mod integration_folder;
mod integration_folder_line;
mod move_integration_folder;

#[cfg(test)]
mod folder_move_refusal_report_tests;
#[cfg(test)]
mod integration_folder_line_tests;

pub use folder_move_refusal_report::folder_move_refusal_report;
pub use integration_folder::integration_folder;
pub use integration_folder_line::integration_folder_line;
pub use move_integration_folder::move_integration_folder;
