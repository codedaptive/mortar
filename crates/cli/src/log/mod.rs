//! `mr log` (SPEC-cli 05, S-114): its filters, the records they keep, their --json encoding, its lines and
//! report, and its runner.

mod encoded_records;
mod log_filters;
mod log_lines;
mod log_report;
mod matching_records;
mod run_log;

#[cfg(test)]
mod encoded_records_tests;
#[cfg(test)]
mod log_filters_tests;
#[cfg(test)]
mod log_lines_tests;
#[cfg(test)]
mod log_report_tests;
#[cfg(test)]
mod matching_records_tests;

pub use encoded_records::encoded_records;
pub use log_filters::log_filters;
pub use log_lines::log_lines;
pub use log_report::log_report;
pub use matching_records::matching_records;
pub use run_log::run_log;
