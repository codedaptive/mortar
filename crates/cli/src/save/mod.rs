//! mr save and mr snapshot: save's runner, the sentence it takes, the run it reads before its lock,
//! the snapshot target a folder saves to (shared by both verbs), the report of a save, of a save
//! that did not happen and of one that failed in a step that may follow its snapshot ref's move; snapshot's runner,
//! the report of a snapshot, of a snapshot that did not happen and of one run in a zone folder.

mod run_save;
mod run_snapshot;
mod save_error_report;
mod save_report;
mod save_run_for;
mod save_sentence;
mod save_step_failure;
mod snapshot_error_report;
mod snapshot_report;
mod snapshot_target_for;
mod zone_folder_refusal;

#[cfg(test)]
mod save_error_report_tests;
#[cfg(test)]
mod save_report_tests;
#[cfg(test)]
mod save_run_for_tests;
#[cfg(test)]
mod save_sentence_tests;
#[cfg(test)]
mod save_step_failure_tests;
#[cfg(test)]
mod snapshot_error_report_tests;
#[cfg(test)]
mod snapshot_report_tests;
#[cfg(test)]
mod snapshot_target_for_tests;
#[cfg(test)]
mod zone_folder_refusal_tests;

pub(crate) use run_save::run_save;
pub(crate) use run_snapshot::run_snapshot;
pub(crate) use save_error_report::save_error_report;
pub(crate) use save_report::save_report;
pub(crate) use save_run_for::save_run_for;
pub(crate) use save_sentence::save_sentence;
pub(crate) use save_step_failure::save_step_failure;
pub(crate) use snapshot_error_report::snapshot_error_report;
pub(crate) use snapshot_report::snapshot_report;
pub(crate) use snapshot_target_for::snapshot_target_for;
pub(crate) use zone_folder_refusal::zone_folder_refusal;
