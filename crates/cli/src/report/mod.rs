//! What a verb reports (S-71): its exit code, its outcome and lines, and the plain and --json forms of it.

mod exit_code;
mod failure_report;
mod is_zero_oid;
mod out_value;
mod prints_nothing;
mod render_json;
mod render_plain;
mod report;

#[cfg(test)]
mod failure_report_tests;
#[cfg(test)]
mod is_zero_oid_tests;
#[cfg(test)]
mod prints_nothing_tests;
#[cfg(test)]
mod render_json_tests;
#[cfg(test)]
mod render_plain_tests;

pub(crate) use exit_code::ExitCode;
pub(crate) use failure_report::failure_report;
pub(crate) use is_zero_oid::is_zero_oid;
pub(crate) use out_value::OutValue;
pub(crate) use prints_nothing::prints_nothing;
pub(crate) use render_json::render_json;
pub(crate) use render_plain::render_plain;
pub(crate) use report::Report;
