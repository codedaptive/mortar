//! `mortar join` (S-72, S-86): the pending folder move detector, the save's inputs, the reports and the runner.

mod find_pending_move;
mod join_outcome;
mod join_refusal_report;
mod join_report;
mod join_save_subject;
mod move_refusal_report;
mod pending_move;
mod read_pending_move;
mod run_join;
mod save_part;

#[cfg(test)]
mod find_pending_move_tests;
#[cfg(test)]
mod join_refusal_report_tests;
#[cfg(test)]
mod join_report_tests;
#[cfg(test)]
mod join_save_subject_tests;
#[cfg(test)]
mod move_refusal_report_tests;
#[cfg(test)]
mod save_part_tests;

pub use find_pending_move::find_pending_move;
pub use join_outcome::JoinOutcome;
pub use join_refusal_report::join_refusal_report;
pub use join_report::join_report;
pub use join_save_subject::join_save_subject;
pub use move_refusal_report::move_refusal_report;
pub use pending_move::PendingMove;
pub use read_pending_move::read_pending_move;
pub use run_join::run_join;
pub use save_part::save_part;
