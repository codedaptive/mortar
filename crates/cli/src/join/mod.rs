//! `mortar join` (S-72, S-86): the pending folder move detector and its finish, the run's setup, the save's
//! inputs, the reports and their parts, and the runner.

mod answer_part;
mod answer_part_value;
mod attempt_refusal;
mod find_pending_move;
mod finish_pending_move;
mod finished_move_part;
mod join_outcome;
mod join_refusal_report;
mod join_report;
mod join_save_subject;
mod move_refusal_report;
mod pending_move;
mod read_pending_move;
mod request_run;
mod request_run_for;
mod run_join;
mod save_part;
mod snapshot_failure;

#[cfg(test)]
mod answer_part_tests;
#[cfg(test)]
mod attempt_refusal_tests;
#[cfg(test)]
mod find_pending_move_tests;
#[cfg(test)]
mod finished_move_part_tests;
#[cfg(test)]
mod join_save_subject_tests;

pub use answer_part::answer_part;
pub use answer_part_value::AnswerPart;
pub use attempt_refusal::attempt_refusal;
pub use find_pending_move::find_pending_move;
pub use finish_pending_move::finish_pending_move;
pub use finished_move_part::finished_move_part;
pub use join_outcome::JoinOutcome;
pub use join_refusal_report::join_refusal_report;
pub use join_report::join_report;
pub use join_save_subject::join_save_subject;
pub use move_refusal_report::move_refusal_report;
pub use pending_move::PendingMove;
pub use read_pending_move::read_pending_move;
pub use request_run::RequestRun;
pub use request_run_for::request_run_for;
pub use run_join::run_join;
pub use save_part::save_part;
pub use snapshot_failure::snapshot_failure;
