//! `mr undo`: the folder's own refs, the choice of the op to undo and its folder step, the
//! oplog caller, the check against a later op, the reports and refusals, and the runner.

mod already_undone_report;
mod choose_undo;
mod failed_undo_report;
mod folder_step;
mod later_op_refusal;
mod nothing_to_undo_report;
mod own_refs;
mod restore_check_report;
mod restore_failed_report;
mod restore_why;
mod reverse_and_step;
mod run_undo;
mod set_index_to_branch;
mod undo_caller;
mod undo_choice;
mod undo_error_report;
mod undo_refusal;
mod undo_report;
mod undo_step;
mod with_restored;

#[cfg(test)]
mod already_undone_report_tests;
#[cfg(test)]
mod choose_undo_tests;
#[cfg(test)]
mod failed_undo_report_tests;
#[cfg(test)]
mod later_op_refusal_tests;
#[cfg(test)]
mod nothing_to_undo_report_tests;
#[cfg(test)]
mod own_refs_tests;
#[cfg(test)]
mod restore_check_report_tests;
#[cfg(test)]
mod restore_failed_report_tests;
#[cfg(test)]
mod restore_why_tests;
#[cfg(test)]
mod undo_caller_tests;
#[cfg(test)]
mod undo_error_report_tests;
#[cfg(test)]
mod undo_refusal_tests;
#[cfg(test)]
mod undo_report_tests;
#[cfg(test)]
mod with_restored_tests;

pub(crate) use already_undone_report::already_undone_report;
pub(crate) use choose_undo::choose_undo;
pub(crate) use failed_undo_report::failed_undo_report;
pub(crate) use later_op_refusal::later_op_refusal;
pub(crate) use nothing_to_undo_report::nothing_to_undo_report;
pub(crate) use own_refs::own_refs;
pub(crate) use restore_check_report::restore_check_report;
pub(crate) use restore_failed_report::restore_failed_report;
pub(crate) use restore_why::restore_why;
pub(crate) use run_undo::run_undo;
pub(crate) use undo_caller::undo_caller;
pub(crate) use undo_choice::UndoChoice;
pub(crate) use undo_error_report::undo_error_report;
pub(crate) use undo_refusal::undo_refusal;
pub(crate) use undo_report::undo_report;
pub(crate) use undo_step::UndoStep;
pub(crate) use with_restored::with_restored;
