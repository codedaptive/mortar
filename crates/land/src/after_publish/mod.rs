//! after_publish: each zone put on or replayed onto the origin a publish left, with its open requests to rebase
//! (SPEC-land 05-after-publish.md).

// The module is named after the function it holds (small-files rule 2), which shares the folder's name.
#[allow(clippy::module_inception)]
mod after_publish;
mod after_publish_error;
mod after_publish_error_display;
mod after_publish_error_source;
mod choose_held;
mod decide_zone_case;
mod move_zone;
mod read_held;
mod read_record_move;
mod read_squash_of;
mod replay_record_ref;
mod replayed_position;
mod requests_to_rebase;
mod squash_of_in;
mod squash_of_read;
mod zone_case;
mod zone_move;
mod zone_move_op;
mod zone_requests;

#[cfg(test)]
mod after_publish_error_display_tests;
#[cfg(test)]
mod after_publish_error_source_tests;
#[cfg(test)]
mod choose_held_tests;
#[cfg(test)]
mod decide_zone_case_tests;
#[cfg(test)]
mod replay_record_ref_tests;
#[cfg(test)]
mod replayed_position_tests;
#[cfg(test)]
mod requests_to_rebase_tests;
#[cfg(test)]
mod squash_of_in_tests;

pub use after_publish::after_publish;
pub use after_publish_error::AfterPublishError;
pub use zone_move::ZoneMove;
pub use zone_requests::ZoneRequests;

pub(crate) use choose_held::choose_held;
pub(crate) use decide_zone_case::decide_zone_case;
pub(crate) use move_zone::move_zone;
pub(crate) use read_held::read_held;
pub(crate) use read_record_move::read_record_move;
pub(crate) use read_squash_of::read_squash_of;
pub(crate) use replay_record_ref::replay_record_ref;
pub(crate) use replayed_position::replayed_position;
pub(crate) use requests_to_rebase::requests_to_rebase;
pub(crate) use squash_of_in::squash_of_in;
pub(crate) use squash_of_read::SquashOfRead;
pub(crate) use zone_case::ZoneCase;
pub(crate) use zone_move_op::zone_move_op;
