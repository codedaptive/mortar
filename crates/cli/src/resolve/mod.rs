//! `mr resolve` (S-50, S-64, S-87): what one run did, its reports, its refusals and the runner.

mod resolve_move_refusal_report;
mod resolve_outcome;
mod resolve_refusal_report;
mod resolve_report;
mod run_resolve;

#[cfg(test)]
mod resolve_move_refusal_report_tests;
#[cfg(test)]
mod resolve_refusal_report_tests;
#[cfg(test)]
mod resolve_report_tests;
#[cfg(test)]
mod run_resolve_tests;

pub(crate) use resolve_move_refusal_report::resolve_move_refusal_report;
pub(crate) use resolve_outcome::ResolveOutcome;
pub(crate) use resolve_refusal_report::resolve_refusal_report;
pub(crate) use resolve_report::resolve_report;
pub(crate) use run_resolve::run_resolve;
