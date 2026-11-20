//! The command line: the verb table, the global flags, and the usage report for a line that cannot be read.

mod after_publish_spec;
mod backup_spec;
mod bisect_spec;
mod disk_spec;
mod find_verb;
mod help_spec;
mod install_spec;
mod lock_need;
mod no_arguments_report;
mod overlap_spec;
mod parse_args;
mod parsed_args;
mod restore_backup_spec;
mod snapshot_spec;
mod sweep_spec;
mod undo_spec;
mod uninstall_spec;
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
mod usage_report_tests;
#[cfg(test)]
mod verbs_setup_tests;
#[cfg(test)]
mod verbs_tests;

pub(crate) use after_publish_spec::AFTER_PUBLISH_SPEC;
pub(crate) use backup_spec::BACKUP_SPEC;
pub(crate) use bisect_spec::BISECT_SPEC;
pub(crate) use disk_spec::DISK_SPEC;
pub(crate) use find_verb::find_verb;
pub(crate) use help_spec::HELP_SPEC;
pub(crate) use install_spec::INSTALL_SPEC;
pub(crate) use lock_need::LockNeed;
pub(crate) use no_arguments_report::no_arguments_report;
pub(crate) use overlap_spec::OVERLAP_SPEC;
pub(crate) use parse_args::parse_args;
pub(crate) use parsed_args::ParsedArgs;
pub(crate) use restore_backup_spec::RESTORE_BACKUP_SPEC;
pub(crate) use snapshot_spec::SNAPSHOT_SPEC;
pub(crate) use sweep_spec::SWEEP_SPEC;
pub(crate) use undo_spec::UNDO_SPEC;
pub(crate) use uninstall_spec::UNINSTALL_SPEC;
pub(crate) use usage_error::UsageError;
pub(crate) use usage_report::usage_report;
pub(crate) use verb_kind::VerbKind;
pub(crate) use verb_spec::VerbSpec;
pub(crate) use verbs::VERBS;
