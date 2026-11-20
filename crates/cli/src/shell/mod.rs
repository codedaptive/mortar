//! The shell every verb runs in: the git floor check it makes first (for every verb but help, install, uninstall
//! and guard-hook), the context it builds, the config it loads, and the one entry point.

mod checks_git_floor;
mod config_refusal;
mod git_floor;
mod git_floor_report;
mod load_config;
mod missing_config_report;
mod run;
mod run_in;
mod shell_context;
mod shell_io;

#[cfg(test)]
mod checks_git_floor_tests;
#[cfg(test)]
mod config_refusal_tests;
#[cfg(test)]
mod git_floor_report_tests;
#[cfg(test)]
mod git_floor_tests;
#[cfg(test)]
mod missing_config_report_tests;
#[cfg(test)]
mod run_in_tests;

pub(crate) use checks_git_floor::checks_git_floor;
pub(crate) use git_floor::git_floor;
pub(crate) use git_floor_report::git_floor_report;
pub(crate) use load_config::load_config;
pub(crate) use missing_config_report::missing_config_report;
pub(crate) use run::run;
pub(crate) use run_in::run_in;
pub(crate) use shell_context::ShellContext;
pub(crate) use shell_io::ShellIo;

pub(crate) use config_refusal::config_refusal;
