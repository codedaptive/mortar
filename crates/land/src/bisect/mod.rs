//! bisect: name the landing that broke the suite (SPEC-land). Holds the top-level bisect, which checks both ends are
//! on origin's first-parent line, then runs the binary search for the first failing landing with each probe
//! running the caller's command in a throwaway detached worktree; the reading of a landing's culprit (its
//! commit, and the change ID and zone from its squash's trailers); and the pieces a bisect run is described by:
//! the caller's test command, each probe and its outcome read from how the command ended, the check that the last
//! passing commit is on the failed one's first-parent line, the report and its failures.

#[allow(clippy::module_inception)]
mod bisect;
mod bisect_answer;
mod bisect_error;
mod bisect_error_display;
mod bisect_error_source;
mod bisect_not_found;
mod bisect_report;
mod bisect_report_display;
mod bisect_verdict;
mod culprit;
mod culprit_error;
mod culprit_error_display;
mod culprit_error_source;
mod culprit_of_landing;
mod on_first_parent_line;
mod probe;
mod probe_commit;
mod probe_display;
mod probe_outcome;
mod probe_outcome_of;
mod search_first_failing;
mod test_command;

#[cfg(test)]
mod bisect_error_display_tests;
#[cfg(test)]
mod bisect_error_source_tests;
#[cfg(test)]
mod bisect_report_display_tests;
#[cfg(test)]
mod culprit_error_source_tests;
#[cfg(test)]
mod culprit_of_landing_tests;
#[cfg(test)]
mod on_first_parent_line_tests;
#[cfg(test)]
mod probe_outcome_of_tests;
#[cfg(test)]
mod search_first_failing_tests;

pub use bisect::bisect;
pub use bisect_answer::BisectAnswer;
pub use bisect_error::BisectError;
pub use bisect_not_found::BisectNotFound;
pub use bisect_report::BisectReport;
pub use bisect_verdict::BisectVerdict;
pub use culprit::Culprit;
pub use culprit_error::CulpritError;
pub use culprit_of_landing::culprit_of_landing;
pub(crate) use on_first_parent_line::on_first_parent_line;
pub use probe::Probe;
pub use probe_outcome::ProbeOutcome;
pub use probe_outcome_of::probe_outcome_of;
pub use search_first_failing::search_first_failing;
pub use test_command::TestCommand;
