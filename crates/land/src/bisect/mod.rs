//! bisect: name the squash that broke the suite (SPEC-land). Holds the binary search for
//! the first failing landing over a probe the caller supplies, and the reading of a landing's culprit (its
//! commit, and the change ID and zone from its squash's trailers).

mod bisect_not_found;
mod bisect_verdict;
mod culprit;
mod culprit_error;
mod culprit_error_display;
mod culprit_error_source;
mod culprit_of_landing;
mod search_first_failing;

#[cfg(test)]
mod culprit_error_display_tests;
#[cfg(test)]
mod culprit_error_source_tests;
#[cfg(test)]
mod culprit_of_landing_tests;
#[cfg(test)]
mod search_first_failing_tests;

pub use bisect_not_found::BisectNotFound;
pub use bisect_verdict::BisectVerdict;
pub use culprit::Culprit;
pub use culprit_error::CulpritError;
pub use culprit_of_landing::culprit_of_landing;
pub use search_first_failing::search_first_failing;
