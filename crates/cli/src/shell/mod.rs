//! The shell every verb runs in: the context it builds, the config it loads, and the one entry point.

mod config_refusal;
mod load_config;
mod missing_config_report;
mod run;
mod shell_context;

#[cfg(test)]
mod config_refusal_tests;
#[cfg(test)]
mod missing_config_report_tests;

pub use load_config::load_config;
pub use missing_config_report::missing_config_report;
pub use run::run;
pub use shell_context::ShellContext;

pub(crate) use config_refusal::config_refusal;
