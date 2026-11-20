//! `mr status`: the state it reports, the reads that fill it, the decisions it makes from that state,
//! its report, the git reads its savings are counted against and its runner, for request folders and attempt folders.

mod attempt_next;
mod attempt_status_report;
mod branch_reads;
mod is_joined;
mod is_saved;
mod joined_from_reads;
mod missing_branch_report;
mod move_stalled;
mod no_work_yet;
mod open_requests_for;
mod overlaps_with;
mod progress_branch;
mod read_branch_reads;
mod read_branch_tips;
mod read_overlaps;
mod read_progress;
mod read_stalled_move;
mod read_status;
mod read_tips;
mod recovered_records;
mod run_status;
mod status_fields;
mod status_lines;
mod status_next;
mod status_outcome;
mod status_phase;
mod status_phase_from_status_state;
mod status_queries;
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
mod joined_from_reads_tests;
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
mod recovered_records_tests;
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
#[cfg(test)]
mod status_queries_tests;
#[cfg(test)]
mod status_report_tests;

pub(crate) use attempt_next::attempt_next;
pub(crate) use attempt_status_report::attempt_status_report;
pub(crate) use branch_reads::BranchReads;
pub(crate) use is_joined::is_joined;
pub(crate) use is_saved::is_saved;
pub(crate) use joined_from_reads::joined_from_reads;
pub(crate) use missing_branch_report::missing_branch_report;
pub(crate) use move_stalled::move_stalled;
pub(crate) use no_work_yet::no_work_yet;
pub(crate) use open_requests_for::open_requests_for;
pub(crate) use overlaps_with::overlaps_with;
pub(crate) use progress_branch::progress_branch;
pub(crate) use read_branch_reads::read_branch_reads;
pub(crate) use read_branch_tips::read_branch_tips;
pub(crate) use read_overlaps::read_overlaps;
pub(crate) use read_progress::read_progress;
pub(crate) use read_stalled_move::read_stalled_move;
pub(crate) use read_status::read_status;
pub(crate) use read_tips::read_tips;
pub(crate) use recovered_records::recovered_records;
pub(crate) use run_status::run_status;
pub(crate) use status_fields::status_fields;
pub(crate) use status_lines::status_lines;
pub(crate) use status_next::status_next;
pub(crate) use status_outcome::status_outcome;
pub(crate) use status_phase::StatusPhase;
pub(crate) use status_queries::status_queries;
pub(crate) use status_report::status_report;
pub(crate) use status_state::StatusState;
