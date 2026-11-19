//! rereview: the re-review diff: what changed in a squash since it was reviewed.

mod plan_rereview;
mod rereview;
mod rereview_error;
mod rereview_error_display;
mod rereview_error_source;
mod rereview_plan;
mod same_change;

#[cfg(test)]
mod plan_rereview_tests;
#[cfg(test)]
mod rereview_error_display_tests;
#[cfg(test)]
mod rereview_error_source_tests;
#[cfg(test)]
mod same_change_tests;

pub(crate) use plan_rereview::plan_rereview;
pub(crate) use rereview_plan::RereviewPlan;
pub(crate) use same_change::same_change;

pub use rereview::rereview;
pub use rereview_error::RereviewError;
