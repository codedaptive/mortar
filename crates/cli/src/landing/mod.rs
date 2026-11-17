//! The cli's words for land's refusals and failures, shared by the landing verbs: squash's, would-merge's, land's,
//! the landing state file's, back-out's and publish's; and the squash those verbs read, named by its commit or by
//! the change ID its squash ref carries.

mod change_conflict_next;
mod land_error_report;
mod landing_refusal;
mod landing_state_error_report;
mod pick_squash_ref;
mod publish_error_report;
mod resolve_squash;
mod squash_error_report;
mod squash_name;
mod squash_pick;
mod squash_pick_report;

#[cfg(test)]
mod change_conflict_next_tests;
#[cfg(test)]
mod land_error_report_tests;
#[cfg(test)]
mod landing_refusal_tests;
#[cfg(test)]
mod landing_state_error_report_tests;
#[cfg(test)]
mod pick_squash_ref_tests;
#[cfg(test)]
mod publish_error_report_tests;
#[cfg(test)]
mod squash_error_report_tests;
#[cfg(test)]
mod squash_pick_report_tests;

pub use change_conflict_next::CHANGE_CONFLICT_NEXT;
pub use land_error_report::land_error_report;
pub use landing_state_error_report::landing_state_error_report;
pub use pick_squash_ref::pick_squash_ref;
pub use publish_error_report::publish_error_report;
pub use resolve_squash::resolve_squash;
pub use squash_error_report::squash_error_report;
pub use squash_name::SquashName;
pub use squash_pick::SquashPick;
pub use squash_pick_report::squash_pick_report;

pub(crate) use landing_refusal::landing_refusal;
