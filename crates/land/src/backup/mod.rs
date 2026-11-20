//! backup: in-flight refs pushed to a backup namespace on the remote, and restored.

mod backup_error;
mod backup_error_display;
mod backup_error_error;
mod backup_prefix;
mod backup_prefixes;
mod backup_ref;
mod backup_type;
mod local_ref;
mod push_plan;
mod restored;
// The module is named after the function it holds (charter: a file is named after its symbol), which
// shares the folder's name.
#[allow(clippy::module_inception)]
mod backup;
mod backup_answer;
mod backup_op;
mod list_backed_up_refs;
mod plan_backup;

mod restore_applied;
mod restore_backup;
mod restore_op;
mod restore_plan;
mod restored_after_fetch;

#[cfg(test)]
mod backup_error_display_tests;
#[cfg(test)]
mod backup_error_error_tests;
#[cfg(test)]
mod backup_prefix_tests;
#[cfg(test)]
mod backup_ref_tests;
#[cfg(test)]
mod local_ref_tests;
#[cfg(test)]
mod backup_answer_tests;
#[cfg(test)]
mod backup_op_tests;
#[cfg(test)]
mod plan_backup_tests;

#[cfg(test)]
mod restore_applied_tests;
#[cfg(test)]
mod restore_op_tests;
#[cfg(test)]
mod restored_after_fetch_tests;

pub use backup_error::BackupError;
pub use backup_type::Backup;
pub use restored::Restored;

pub(crate) use backup_prefix::backup_prefix;
pub(crate) use backup_prefixes::BACKUP_PREFIXES;
pub(crate) use backup_ref::backup_ref;
pub(crate) use local_ref::local_ref;
pub(crate) use push_plan::PushPlan;
pub use backup::backup;
pub(crate) use backup_answer::backup_answer;
pub(crate) use backup_op::backup_op;
pub(crate) use list_backed_up_refs::list_backed_up_refs;
pub(crate) use plan_backup::plan_backup;

pub use restore_backup::restore_backup;
pub(crate) use restore_applied::restore_applied;
pub(crate) use restore_op::restore_op;
pub(crate) use restore_plan::restore_plan;
pub(crate) use restored_after_fetch::restored_after_fetch;
