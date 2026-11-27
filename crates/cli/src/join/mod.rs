//! `mr join`: the pending folder move detector (a rebase's, a joined partner request's or an undo's), the base its
//! folder record must hold and its finish, the folder record's rewrite, the run's setup, the save's inputs, the
//! reports and their parts, and the runner.

mod answer_part;
mod answer_part_value;
mod attempt_refusal;
mod base_answer_part;
mod find_pending_move;
mod finish_pending_move;
mod finished_move_part;
mod follow_answer;
mod join_outcome;
mod join_refusal_report;
mod join_report;
mod join_save_subject;
mod joined_zone_base;
mod move_cause;
mod move_refusal_report;
mod moved_before;
mod partners_conflicted_report;
mod pending_base;
mod pending_move;
mod read_pending_move;
mod request_run;
mod rewrite_record_to;
mod request_run_for;
mod run_join;
mod save_from_formatted;
mod save_part;
mod snapshot_failure;
mod snapshot_moved;

#[cfg(test)]
mod answer_part_tests;
#[cfg(test)]
mod attempt_refusal_tests;
#[cfg(test)]
mod base_answer_part_tests;
#[cfg(test)]
mod find_pending_move_tests;
#[cfg(test)]
mod finished_move_part_tests;
#[cfg(test)]
mod join_refusal_report_tests;
#[cfg(test)]
mod join_report_tests;
#[cfg(test)]
mod join_save_subject_tests;
#[cfg(test)]
mod joined_zone_base_tests;
#[cfg(test)]
mod move_refusal_report_tests;
#[cfg(test)]
mod moved_before_tests;
#[cfg(test)]
mod partners_conflicted_report_tests;
#[cfg(test)]
mod pending_base_tests;
#[cfg(test)]
mod request_run_for_tests;
#[cfg(test)]
mod save_part_tests;
#[cfg(test)]
mod snapshot_failure_tests;
#[cfg(test)]
mod snapshot_moved_tests;

pub(crate) use answer_part::answer_part;
pub(crate) use answer_part_value::AnswerPart;
pub(crate) use attempt_refusal::attempt_refusal;
pub(crate) use base_answer_part::base_answer_part;
pub(crate) use find_pending_move::find_pending_move;
pub(crate) use finish_pending_move::finish_pending_move;
pub(crate) use finished_move_part::finished_move_part;
pub(crate) use join_outcome::JoinOutcome;
pub(crate) use join_refusal_report::join_refusal_report;
pub(crate) use join_report::join_report;
pub(crate) use join_save_subject::join_save_subject;
pub(crate) use joined_zone_base::joined_zone_base;
pub(crate) use move_cause::MoveCause;
pub(crate) use move_refusal_report::move_refusal_report;
pub(crate) use moved_before::moved_before;
pub(crate) use partners_conflicted_report::partners_conflicted_report;
pub(crate) use pending_base::pending_base;
pub(crate) use pending_move::PendingMove;
pub(crate) use read_pending_move::read_pending_move;
pub(crate) use request_run::RequestRun;
pub(crate) use request_run_for::request_run_for;
pub(crate) use rewrite_record_to::rewrite_record_to;
pub(crate) use run_join::run_join;
pub(crate) use save_from_formatted::save_from_formatted;
pub(crate) use save_part::save_part;
pub(crate) use snapshot_failure::snapshot_failure;
pub(crate) use snapshot_moved::snapshot_moved;
