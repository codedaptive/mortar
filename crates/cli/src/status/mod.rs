//! `mortar status` (spec 02): the state it reports, the reads that fill it, the decisions it makes from that state,
//! its report and its runner.

mod is_joined;
mod missing_branch_report;
mod move_stalled;
mod open_requests_for;
mod overlaps_with;
mod read_branch_tips;
mod read_overlaps;
mod read_stalled_move;
mod read_status;
mod run_status;
mod status_fields;
mod status_lines;
mod status_next;
mod status_outcome;
mod status_report;
mod status_state;

#[cfg(test)]
mod is_joined_tests;
#[cfg(test)]
mod missing_branch_report_tests;
#[cfg(test)]
mod move_stalled_tests;
#[cfg(test)]
mod open_requests_for_tests;
#[cfg(test)]
mod overlaps_with_tests;
#[cfg(test)]
mod status_fields_tests;
#[cfg(test)]
mod status_lines_tests;
#[cfg(test)]
mod status_next_tests;
#[cfg(test)]
mod status_outcome_tests;

pub use is_joined::is_joined;
pub use missing_branch_report::missing_branch_report;
pub use move_stalled::move_stalled;
pub use open_requests_for::open_requests_for;
pub use overlaps_with::overlaps_with;
pub use read_branch_tips::read_branch_tips;
pub use read_overlaps::read_overlaps;
pub use read_stalled_move::read_stalled_move;
pub use read_status::read_status;
pub use run_status::run_status;
pub use status_fields::status_fields;
pub use status_lines::status_lines;
pub use status_next::status_next;
pub use status_outcome::status_outcome;
pub use status_report::status_report;
pub use status_state::StatusState;
