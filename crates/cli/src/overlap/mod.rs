//! `mortar overlap <zone>` (SPEC-cli 05): its words, its reports and its runner.

mod overlap_error_report;
mod overlap_report;
mod parse_overlap_args;
mod run_overlap;

#[cfg(test)]
mod overlap_error_report_tests;
#[cfg(test)]
mod overlap_report_tests;
#[cfg(test)]
mod parse_overlap_args_tests;

pub use overlap_error_report::overlap_error_report;
pub use overlap_report::overlap_report;
pub use parse_overlap_args::parse_overlap_args;
pub use run_overlap::run_overlap;
