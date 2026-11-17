//! What `mr disk` and `mr sweep` share: the zone list read from the config (never empty, S-121 (2)), the
//! refusal of a config with no zone, and a folders error in the cli's words under the verb it came from.

mod config_zones;
mod folders_error_report;
mod no_zones_report;

#[cfg(test)]
mod config_zones_tests;
#[cfg(test)]
mod folders_error_report_tests;
#[cfg(test)]
mod no_zones_report_tests;

pub use config_zones::config_zones;
pub use folders_error_report::folders_error_report;
pub use no_zones_report::no_zones_report;
