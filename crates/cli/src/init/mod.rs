//! `mr init` (S-70): its arguments, what it decides against the config already there, and its report.

mod decide_init;
mod init_decision;
mod init_flag_value;
mod init_options;
mod init_report;
mod parse_init_args;
mod parse_zone_flag;
mod run_init;

#[cfg(test)]
mod decide_init_tests;
#[cfg(test)]
mod init_flag_value_tests;
#[cfg(test)]
mod init_report_tests;
#[cfg(test)]
mod parse_init_args_tests;
#[cfg(test)]
mod parse_zone_flag_tests;

pub use decide_init::decide_init;
pub use init_decision::InitDecision;
pub use init_options::InitOptions;
pub use init_report::init_report;
pub use parse_init_args::parse_init_args;
pub use run_init::run_init;

pub(crate) use init_flag_value::init_flag_value;
pub(crate) use parse_zone_flag::parse_zone_flag;
