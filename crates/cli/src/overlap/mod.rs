//! `mr overlap <zone>`: its words, its reports and its runner.

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

pub(crate) use overlap_error_report::overlap_error_report;
pub(crate) use overlap_report::overlap_report;
pub(crate) use parse_overlap_args::parse_overlap_args;
pub(crate) use run_overlap::run_overlap;
