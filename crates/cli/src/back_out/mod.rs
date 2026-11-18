//! `mr back-out` (SPEC-cli 06): its report in the cli's own words, and its runner.

mod back_out_report;
mod run_back_out;

#[cfg(test)]
mod back_out_report_tests;

pub(crate) use back_out_report::back_out_report;
pub(crate) use run_back_out::run_back_out;
