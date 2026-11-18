//! `mr sweep` (SPEC-cli 05): its words, its reports and its runner.

mod parse_sweep_args;
mod run_sweep;
mod sweep_args;
mod sweep_build_failure_report;
mod sweep_report;

#[cfg(test)]
mod parse_sweep_args_tests;
#[cfg(test)]
mod sweep_build_failure_report_tests;
#[cfg(test)]
mod sweep_report_tests;

pub(crate) use parse_sweep_args::parse_sweep_args;
pub(crate) use run_sweep::run_sweep;
pub(crate) use sweep_args::SweepArgs;
pub(crate) use sweep_build_failure_report::sweep_build_failure_report;
pub(crate) use sweep_report::sweep_report;
