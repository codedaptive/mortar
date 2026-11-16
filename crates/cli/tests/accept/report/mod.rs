//! The acceptance report: the lines a step records, its verdict, the plain-text report and where it is written.

mod block;
mod block_verdict;
mod checks;
mod checks_check;
mod checks_lines;
mod checks_log;
mod checks_new;
mod drive_passed;
mod format_report;
mod line;
mod report_path;
mod verdict;
mod write_report;

#[cfg(test)]
mod block_verdict_tests;
#[cfg(test)]
mod checks_check_tests;
#[cfg(test)]
mod checks_new_tests;
#[cfg(test)]
mod drive_passed_tests;
#[cfg(test)]
mod format_report_tests;
#[cfg(test)]
mod report_path_tests;

pub use block::Block;
pub use block_verdict::block_verdict;
pub use checks::Checks;
pub use drive_passed::drive_passed;
pub use format_report::format_report;
pub use line::Line;
pub use report_path::report_path;
pub use verdict::Verdict;
pub use write_report::write_report;
