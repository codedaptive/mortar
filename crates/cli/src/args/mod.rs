//! The command line: the verb table, the global flags, and the usage report for a line that cannot be read.

mod find_verb;
mod lock_need;
mod parse_args;
mod parsed_args;
mod usage_error;
mod usage_report;
mod verb_kind;
mod verb_spec;
mod verbs;

#[cfg(test)]
mod find_verb_tests;
#[cfg(test)]
mod parse_args_tests;
#[cfg(test)]
mod usage_report_tests;
#[cfg(test)]
mod verbs_tests;

pub use find_verb::find_verb;
pub use lock_need::LockNeed;
pub use parse_args::parse_args;
pub use parsed_args::ParsedArgs;
pub use usage_error::UsageError;
pub use usage_report::usage_report;
pub use verb_kind::VerbKind;
pub use verb_spec::VerbSpec;
pub use verbs::VERBS;
