//! `mr after-publish`: its words, land's zone moves and refusals in the cli's own words, its report, and its
//! runner, which brings each named zone's folder to its zone branch's tip through zone_folder_move.

mod after_publish_args;
mod after_publish_error_report;
mod after_publish_failure;
mod after_publish_report;
mod named_zone;
mod parse_after_publish_args;
mod read_named_zones;
mod repeat_command;
mod run_after_publish;
mod zone_move_lines;

#[cfg(test)]
mod after_publish_error_report_tests;
#[cfg(test)]
mod after_publish_failure_tests;
#[cfg(test)]
mod after_publish_report_tests;
#[cfg(test)]
mod parse_after_publish_args_tests;
#[cfg(test)]
mod repeat_command_tests;
#[cfg(test)]
mod zone_move_lines_tests;

pub(crate) use after_publish_args::AfterPublishArgs;
pub(crate) use after_publish_error_report::after_publish_error_report;
pub(crate) use after_publish_failure::after_publish_failure;
pub(crate) use after_publish_report::after_publish_report;
pub(crate) use named_zone::NamedZone;
pub(crate) use parse_after_publish_args::parse_after_publish_args;
pub(crate) use read_named_zones::read_named_zones;
pub(crate) use repeat_command::repeat_command;
pub(crate) use run_after_publish::run_after_publish;
pub(crate) use zone_move_lines::zone_move_lines;
