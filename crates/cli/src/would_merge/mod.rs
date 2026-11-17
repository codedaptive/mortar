//! `mortar would-merge` (SPEC-cli 06): its words, its report in the cli's own words, and its runner.

mod parse_would_merge_args;
mod run_would_merge;
mod would_merge_report;

#[cfg(test)]
mod parse_would_merge_args_tests;
#[cfg(test)]
mod would_merge_report_tests;

pub use parse_would_merge_args::parse_would_merge_args;
pub use run_would_merge::run_would_merge;
pub use would_merge_report::would_merge_report;
