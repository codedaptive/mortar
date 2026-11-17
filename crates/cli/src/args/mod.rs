//! The command line: the verb table, the global flags, and the usage report for a line that cannot be read.

mod after_publish_spec;
mod backup_spec;
mod bisect_spec;
mod disk_spec;
mod find_verb;
mod lock_need;
mod no_arguments_report;
mod overlap_spec;
mod parse_args;
mod parsed_args;
mod restore_backup_spec;
mod snapshot_spec;
mod sweep_spec;
mod usage_error;
mod usage_report;
mod verb_kind;
mod verb_spec;
mod verbs;

#[cfg(test)]
mod find_verb_tests;
#[cfg(test)]
mod no_arguments_report_tests;
#[cfg(test)]
mod parse_args_tests;
#[cfg(test)]
mod verbs_tests;

pub use after_publish_spec::AFTER_PUBLISH_SPEC;
pub use backup_spec::BACKUP_SPEC;
pub use bisect_spec::BISECT_SPEC;
pub use disk_spec::DISK_SPEC;
pub use find_verb::find_verb;
pub use lock_need::LockNeed;
pub use no_arguments_report::no_arguments_report;
pub use overlap_spec::OVERLAP_SPEC;
pub use parse_args::parse_args;
pub use parsed_args::ParsedArgs;
pub use restore_backup_spec::RESTORE_BACKUP_SPEC;
pub use snapshot_spec::SNAPSHOT_SPEC;
pub use sweep_spec::SWEEP_SPEC;
pub use usage_error::UsageError;
pub use usage_report::usage_report;
pub use verb_kind::VerbKind;
pub use verb_spec::VerbSpec;
pub use verbs::VERBS;
