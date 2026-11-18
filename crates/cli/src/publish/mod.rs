//! `mr publish` (SPEC-cli 06): its report in the cli's own words, and its runner.

mod publish_report;
mod run_publish;

#[cfg(test)]
mod publish_report_tests;

pub(crate) use publish_report::publish_report;
pub(crate) use run_publish::run_publish;
