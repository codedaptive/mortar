//! squash: a zone's changes squashed onto origin as one commit at `refs/mortar/squash/<zone>`, with its
//! message, trailers, change-ID choice and repeat decision, whether that squash merges into integration, and a
//! squash commit read back as a Squash.

mod check_attr_source;
mod choose_change_id;
mod commit_subject;
mod land_refs;
mod last_trailer_value;
mod prior_squash;
mod read_prior_squash;
mod read_squash;
mod read_tip;
mod request_commits;
mod same_squash;
// The module is named after the function it holds, which shares the folder's name.
#[allow(clippy::module_inception)]
mod squash;
mod squash_answer;
mod squash_applied;
mod squash_error;
mod squash_error_display;
mod squash_error_error;
mod squash_from_parts;
mod squash_message;
mod squash_message_requests;
mod squash_of_key;
mod squash_op;
mod squash_ref;
mod squash_request;
mod squash_requests;
mod squash_trailer_pairs;
mod squash_type;
mod would_merge;
mod would_merge_answer;
mod would_merge_from;

#[cfg(test)]
mod choose_change_id_tests;
#[cfg(test)]
mod commit_subject_tests;
#[cfg(test)]
mod last_trailer_value_tests;
#[cfg(test)]
mod same_squash_tests;
#[cfg(test)]
mod squash_applied_tests;
#[cfg(test)]
mod squash_error_display_tests;
#[cfg(test)]
mod squash_error_error_tests;
#[cfg(test)]
mod squash_from_parts_tests;
#[cfg(test)]
mod squash_message_requests_tests;
#[cfg(test)]
mod squash_message_tests;
#[cfg(test)]
mod squash_ref_tests;
#[cfg(test)]
mod squash_requests_tests;
#[cfg(test)]
mod would_merge_from_tests;

pub use land_refs::LandRefs;
pub use read_squash::read_squash;
pub use squash::squash;
pub use squash_answer::SquashAnswer;
pub use squash_error::SquashError;
pub use squash_type::Squash;
pub use would_merge::would_merge;
pub use would_merge_answer::WouldMerge;

pub(crate) use check_attr_source::check_attr_source;
pub(crate) use choose_change_id::choose_change_id;
pub(crate) use commit_subject::commit_subject;
pub(crate) use last_trailer_value::last_trailer_value;
pub(crate) use prior_squash::PriorSquash;
pub(crate) use read_prior_squash::read_prior_squash;
pub(crate) use read_tip::read_tip;
pub(crate) use request_commits::request_commits;
pub(crate) use same_squash::same_squash;
pub(crate) use squash_applied::squash_applied;
pub(crate) use squash_from_parts::squash_from_parts;
pub(crate) use squash_message::squash_message;
pub(crate) use squash_message_requests::squash_message_requests;
pub(crate) use squash_of_key::MORTAR_SQUASH_OF_KEY;
pub(crate) use squash_op::squash_op;
pub(crate) use squash_ref::squash_ref;
pub(crate) use squash_request::SquashRequest;
pub(crate) use squash_requests::squash_requests;
pub(crate) use squash_trailer_pairs::squash_trailer_pairs;
pub(crate) use would_merge_from::would_merge_from;
