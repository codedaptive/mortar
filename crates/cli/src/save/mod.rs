//! mortar save: the verb's runner, the sentence it takes (S-41), the snapshot target its folder saves to (S-69),
//! the report of a save, and the report of a save that did not happen.

mod run_save;
mod save_error_report;
mod save_report;
mod save_sentence;
mod snapshot_target_for;

#[cfg(test)]
mod save_report_tests;
#[cfg(test)]
mod save_sentence_tests;
#[cfg(test)]
mod snapshot_target_for_tests;

pub use run_save::run_save;
pub use save_error_report::save_error_report;
pub use save_report::save_report;
pub use save_sentence::save_sentence;
pub use snapshot_target_for::snapshot_target_for;
