//! `mortar disk` (SPEC-cli 05): its report and its runner.

mod disk_report;
mod run_disk;

#[cfg(test)]
mod disk_report_tests;

pub use disk_report::disk_report;
pub use run_disk::run_disk;
