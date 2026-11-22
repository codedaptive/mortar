//! `mr publish [--no-push]`: its words, its report in the cli's own words, and its runner.

mod parse_publish_args;
mod publish_report;
mod run_publish;

#[cfg(test)]
mod parse_publish_args_tests;
#[cfg(test)]
mod publish_report_tests;

pub(crate) use parse_publish_args::parse_publish_args;
pub(crate) use publish_report::publish_report;
pub(crate) use run_publish::run_publish;
