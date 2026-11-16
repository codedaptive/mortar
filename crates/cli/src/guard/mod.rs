//! `mortar guard` (SPEC-cli 06): its words, the config's branches as land's LandRefs, whether the guard is in place,
//! its reports and refusals in the cli's own words, and the runners of install and status.

mod foreign_hook_report;
mod guard_command;
mod guard_error_report;
mod guard_fields;
mod guard_in_place;
mod guard_installed_report;
mod guard_refs_for;
mod guard_state_report;
mod parse_guard_args;
mod run_guard;
mod run_guard_install;
mod run_guard_status;

#[cfg(test)]
mod foreign_hook_report_tests;
#[cfg(test)]
mod guard_error_report_tests;
#[cfg(test)]
mod guard_fields_tests;
#[cfg(test)]
mod guard_in_place_tests;
#[cfg(test)]
mod guard_installed_report_tests;
#[cfg(test)]
mod guard_refs_for_tests;
#[cfg(test)]
mod guard_state_report_tests;
#[cfg(test)]
mod parse_guard_args_tests;

pub use foreign_hook_report::foreign_hook_report;
pub use guard_command::GuardCommand;
pub use guard_error_report::guard_error_report;
pub use guard_fields::guard_fields;
pub use guard_in_place::guard_in_place;
pub use guard_installed_report::guard_installed_report;
pub use guard_refs_for::guard_refs_for;
pub use guard_state_report::guard_state_report;
pub use parse_guard_args::parse_guard_args;
pub use run_guard::run_guard;
pub use run_guard_install::run_guard_install;
pub use run_guard_status::run_guard_status;
