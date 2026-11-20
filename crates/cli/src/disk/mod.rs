//! `mr disk`: its report and its runner.

mod disk_report;
mod run_disk;

#[cfg(test)]
mod disk_report_tests;

pub(crate) use disk_report::disk_report;
pub(crate) use run_disk::run_disk;
