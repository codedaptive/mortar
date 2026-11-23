//! `mr commit-files`: its words, its default message, the op that moves a zone branch, the pending marker of a zone
//! folder's run, its reports and refusals in the cli's own words, and its runners for a zone folder and the
//! integration folder.

mod commit_files_error_report;
mod commit_files_folder_refusal;
mod commit_files_report;
mod commit_paths_error_report;
mod default_commit_message;
mod moved_since_read_refusal;
mod parse_commit_files_args;
mod parse_pending_commit;
mod pending_commit;
mod pending_commit_text;
mod read_pending_commit;
mod run_commit_files;
mod run_commit_files_integration;
mod run_commit_files_zone;
mod stopped_run_refusal;
mod write_pending_commit;
mod zone_commit_op;
mod zone_off_tip_refusal;

#[cfg(test)]
mod commit_files_error_report_tests;
#[cfg(test)]
mod commit_files_folder_refusal_tests;
#[cfg(test)]
mod commit_files_report_tests;
#[cfg(test)]
mod commit_paths_error_report_tests;
#[cfg(test)]
mod default_commit_message_tests;
#[cfg(test)]
mod moved_since_read_refusal_tests;
#[cfg(test)]
mod parse_commit_files_args_tests;
#[cfg(test)]
mod parse_pending_commit_tests;
#[cfg(test)]
mod pending_commit_text_tests;
#[cfg(test)]
mod stopped_run_refusal_tests;
#[cfg(test)]
mod zone_commit_op_tests;
#[cfg(test)]
mod zone_off_tip_refusal_tests;

pub(crate) use commit_files_error_report::commit_files_error_report;
pub(crate) use commit_files_folder_refusal::commit_files_folder_refusal;
pub(crate) use commit_files_report::commit_files_report;
pub(crate) use commit_paths_error_report::commit_paths_error_report;
pub(crate) use default_commit_message::default_commit_message;
pub(crate) use moved_since_read_refusal::moved_since_read_refusal;
pub(crate) use parse_commit_files_args::parse_commit_files_args;
pub(crate) use parse_pending_commit::parse_pending_commit;
pub(crate) use pending_commit::PendingCommit;
pub(crate) use pending_commit_text::pending_commit_text;
pub(crate) use run_commit_files::run_commit_files;
pub(crate) use stopped_run_refusal::stopped_run_refusal;
pub(crate) use zone_commit_op::zone_commit_op;
pub(crate) use zone_off_tip_refusal::zone_off_tip_refusal;
