//! `mr need <path>`: the path and branch checks before the lock, the reports of a need, of a path the
//! repository does not hold and of a need folders refused, the next step added to a finished need, and the runner.

mod missing_path_report;
mod need_error_report;
mod need_report;
mod need_with_next;
mod read_need_path;
mod run_need;

#[cfg(test)]
mod missing_path_report_tests;
#[cfg(test)]
mod need_error_report_tests;
#[cfg(test)]
mod need_report_tests;
#[cfg(test)]
mod need_with_next_tests;

pub(crate) use missing_path_report::missing_path_report;
pub(crate) use need_error_report::need_error_report;
pub(crate) use need_report::need_report;
pub(crate) use need_with_next::need_with_next;
pub(crate) use read_need_path::read_need_path;
pub(crate) use run_need::run_need;
