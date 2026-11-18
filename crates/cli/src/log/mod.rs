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
#[cfg(test)]
mod run_log_tests;

pub(crate) use encoded_records::encoded_records;
pub(crate) use log_filters::log_filters;
pub(crate) use log_lines::log_lines;
pub(crate) use log_report::log_report;
pub(crate) use matching_records::matching_records;
pub(crate) use run_log::run_log;
