//! `mr guard` (SPEC-cli 06): its words, the config's branches as land's LandRefs, whether the guard is in place,
//! its reports and refusals in the cli's own words, and the runners of install and status.

mod foreign_hook_report;
mod guard_command;
mod guard_error_report;
mod guard_fields;
mod guard_in_place;
mod guard_installed_report;
mod guard_refs_for;
mod guard_state_report;
mod hooks_path_report;
mod install_outcome;
mod parse_guard_args;
mod run_guard;
mod run_guard_install;
mod run_guard_status;
mod with_path_warning;

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
mod hooks_path_report_tests;
#[cfg(test)]
mod install_outcome_tests;
#[cfg(test)]
mod parse_guard_args_tests;
#[cfg(test)]
mod with_path_warning_tests;

pub(crate) use foreign_hook_report::foreign_hook_report;
pub(crate) use guard_command::GuardCommand;
pub(crate) use guard_error_report::guard_error_report;
pub(crate) use guard_fields::guard_fields;
pub(crate) use guard_in_place::guard_in_place;
pub(crate) use guard_installed_report::guard_installed_report;
pub(crate) use guard_refs_for::guard_refs_for;
pub(crate) use guard_state_report::guard_state_report;
pub(crate) use hooks_path_report::hooks_path_report;
pub(crate) use install_outcome::install_outcome;
pub(crate) use parse_guard_args::parse_guard_args;
pub(crate) use run_guard::run_guard;
pub(crate) use run_guard_install::run_guard_install;
pub(crate) use run_guard_status::run_guard_status;
pub(crate) use with_path_warning::with_path_warning;
