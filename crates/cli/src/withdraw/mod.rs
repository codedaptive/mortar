//! `mr withdraw`: its words, its report in the cli's own words, and its runner.

mod parse_withdraw_args;
mod run_withdraw;
mod withdraw_report;

#[cfg(test)]
mod parse_withdraw_args_tests;
#[cfg(test)]
mod withdraw_report_tests;

pub(crate) use parse_withdraw_args::parse_withdraw_args;
pub(crate) use run_withdraw::run_withdraw;
pub(crate) use withdraw_report::withdraw_report;
