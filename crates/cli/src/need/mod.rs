//! `mortar need <path>` (spec 02): the path check before the lock, the reports of a need, of a path the repository
//! does not hold and of a need folders refused, and the runner.

mod missing_path_report;
mod need_error_report;
mod need_report;
mod read_need_path;
mod run_need;

#[cfg(test)]
mod missing_path_report_tests;
#[cfg(test)]
mod need_error_report_tests;
#[cfg(test)]
mod need_report_tests;

pub use missing_path_report::missing_path_report;
pub use need_error_report::need_error_report;
pub use need_report::need_report;
pub use read_need_path::read_need_path;
pub use run_need::run_need;
