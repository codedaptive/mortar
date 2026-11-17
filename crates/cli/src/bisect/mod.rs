//! `mr bisect` (SPEC-cli 06): its words, its report and land's refusals in the cli's own words, and its runner.

mod bisect_args;
mod bisect_error_report;
mod bisect_report;
mod culprit_line;
mod parse_bisect_args;
mod probe_line;
mod run_bisect;

#[cfg(test)]
mod bisect_error_report_tests;
#[cfg(test)]
mod bisect_report_tests;
#[cfg(test)]
mod culprit_line_tests;
#[cfg(test)]
mod parse_bisect_args_tests;
#[cfg(test)]
mod probe_line_tests;

pub use bisect_args::BisectArgs;
pub use bisect_error_report::bisect_error_report;
pub use bisect_report::bisect_report;
pub use culprit_line::culprit_line;
pub use parse_bisect_args::parse_bisect_args;
pub use probe_line::probe_line;
pub use run_bisect::run_bisect;
