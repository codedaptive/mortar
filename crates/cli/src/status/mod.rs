//! `mortar status` (spec 02): the state it reports, the reads that fill it, the decisions it makes from that state,
//! its report and its runner, for request folders and attempt folders (S-127(1)).

mod attempt_next;
mod attempt_status_report;
mod is_joined;
mod is_saved;
mod missing_branch_report;
mod move_stalled;
mod no_work_yet;
mod open_requests_for;
mod overlaps_with;
mod progress_branch;
mod read_branch_tips;
mod read_overlaps;
mod read_progress;
mod read_stalled_move;
mod read_status;
mod read_tips;
mod run_status;
mod status_fields;
mod status_lines;
mod status_next;
mod status_outcome;
mod status_phase;
mod status_phase_from_status_state;
mod status_report;
mod status_state;

#[cfg(test)]
mod attempt_next_tests;
#[cfg(test)]
mod attempt_status_report_tests;
#[cfg(test)]
mod is_joined_tests;
#[cfg(test)]
mod is_saved_tests;
#[cfg(test)]
mod missing_branch_report_tests;
#[cfg(test)]
mod move_stalled_tests;
#[cfg(test)]
mod no_work_yet_tests;
#[cfg(test)]
mod open_requests_for_tests;
#[cfg(test)]
mod overlaps_with_tests;
#[cfg(test)]
mod progress_branch_tests;
#[cfg(test)]
mod status_fields_tests;
#[cfg(test)]
mod status_lines_tests;
#[cfg(test)]
mod status_next_tests;
#[cfg(test)]
mod status_outcome_tests;
#[cfg(test)]
mod status_phase_from_status_state_tests;

pub use attempt_next::attempt_next;
pub use attempt_status_report::attempt_status_report;
pub use is_joined::is_joined;
pub use is_saved::is_saved;
pub use missing_branch_report::missing_branch_report;
pub use move_stalled::move_stalled;
pub use no_work_yet::no_work_yet;
pub use open_requests_for::open_requests_for;
pub use overlaps_with::overlaps_with;
pub use progress_branch::progress_branch;
pub use read_branch_tips::read_branch_tips;
pub use read_overlaps::read_overlaps;
pub use read_progress::read_progress;
pub use read_stalled_move::read_stalled_move;
pub use read_status::read_status;
pub use read_tips::read_tips;
pub use run_status::run_status;
pub use status_fields::status_fields;
pub use status_lines::status_lines;
pub use status_next::status_next;
pub use status_outcome::status_outcome;
pub use status_phase::StatusPhase;
pub use status_report::status_report;
pub use status_state::StatusState;
