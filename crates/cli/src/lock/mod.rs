//! The oplog lock a verb takes, what a refused lock reports, and running an act under it after recover (S-16).

mod lock_name_for;
mod lock_report;
mod locked_act;

#[cfg(test)]
mod lock_name_for_tests;
#[cfg(test)]
mod lock_report_tests;

pub(crate) use lock_name_for::lock_name_for;
pub use lock_report::lock_report;
pub(crate) use locked_act::locked_act;
