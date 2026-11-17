//! `mr land` (SPEC-cli 06): its words, its report in the cli's own words, and its runner. The module shares
//! its name with the land crate, so code here names the crate `::land`.

mod land_report;
mod parse_land_args;
mod run_land;

#[cfg(test)]
mod land_report_tests;
#[cfg(test)]
mod parse_land_args_tests;

pub use land_report::land_report;
pub use parse_land_args::parse_land_args;
pub use run_land::run_land;
