//! `mr squash` (SPEC-cli 06): its words, its report in the cli's own words, and its runner.

mod parse_squash_args;
mod run_squash;
mod squash_report;

#[cfg(test)]
mod parse_squash_args_tests;
#[cfg(test)]
mod squash_report_tests;

pub use parse_squash_args::parse_squash_args;
pub use run_squash::run_squash;
pub use squash_report::squash_report;
