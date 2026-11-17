//! The integration folder: the worktree on the integration branch, found from git, moved after a verb moves that
//! branch, a move a stopped run left undone finished by the next land, back-out or publish (S-137), and its report
//! lines and refusal in the cli's own words.

mod finish_stopped_move;
mod finished_move;
mod folder_move_refusal_report;
mod integration_folder;
mod integration_folder_line;
mod integration_move_stopped;
mod last_integration_old;
mod move_block;
mod move_integration_folder;
mod note_finished_move;

#[cfg(test)]
mod folder_move_refusal_report_tests;
#[cfg(test)]
mod integration_folder_line_tests;
#[cfg(test)]
mod integration_move_stopped_tests;
#[cfg(test)]
mod last_integration_old_tests;
#[cfg(test)]
mod note_finished_move_tests;

pub use finish_stopped_move::finish_stopped_move;
pub use finished_move::FinishedMove;
pub use folder_move_refusal_report::folder_move_refusal_report;
pub use integration_folder::integration_folder;
pub use integration_folder_line::integration_folder_line;
pub use integration_move_stopped::integration_move_stopped;
pub use last_integration_old::last_integration_old;
pub use move_block::MoveBlock;
pub use move_integration_folder::move_integration_folder;
pub use note_finished_move::note_finished_move;
