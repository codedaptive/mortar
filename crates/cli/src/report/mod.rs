//! What a verb reports: its exit code, its outcome and lines, and the plain and --json forms of it; the report of
//! save, join or resolve failing in a step that may follow the snapshot ref's move, and the placing of
//! moved_before's line first in a report's lines, which all three share; and the refusal of a verb whose folder's
//! HEAD is unborn or detached, which snapshot, join and resolve share.

mod exit_code;
mod failure_report;
mod head_refusal;
mod is_zero_oid;
mod out_value;
mod prints_nothing;
mod render_json;
mod render_plain;
mod report;
mod snapshot_error_detail;
mod snapshot_step_failure;
mod with_moved_first;

#[cfg(test)]
mod failure_report_tests;
#[cfg(test)]
mod head_refusal_tests;
#[cfg(test)]
mod is_zero_oid_tests;
#[cfg(test)]
mod prints_nothing_tests;
#[cfg(test)]
mod render_json_tests;
#[cfg(test)]
mod render_plain_tests;
#[cfg(test)]
mod snapshot_error_detail_tests;
#[cfg(test)]
mod snapshot_step_failure_tests;
#[cfg(test)]
mod with_moved_first_tests;

pub(crate) use exit_code::ExitCode;
pub(crate) use failure_report::failure_report;
pub(crate) use head_refusal::head_refusal;
pub(crate) use is_zero_oid::is_zero_oid;
pub(crate) use out_value::OutValue;
pub(crate) use prints_nothing::prints_nothing;
pub(crate) use render_json::render_json;
pub(crate) use render_plain::render_plain;
pub(crate) use report::Report;
use snapshot_error_detail::snapshot_error_detail;
pub(crate) use snapshot_step_failure::snapshot_step_failure;
pub(crate) use with_moved_first::with_moved_first;
