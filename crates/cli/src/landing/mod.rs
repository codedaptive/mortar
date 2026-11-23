//! The cli's words for land's refusals and failures, shared by the landing verbs: squash's, would-merge's, land's,
//! the landing state file's, back-out's, publish's and withdraw's; the squash those verbs read, named by its commit
//! or by the change ID its squash ref carries; and the check that a zone word is a zone the config names.

mod change_conflict_next;
mod known_zone;
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
mod withdraw_error_report;

#[cfg(test)]
mod change_conflict_next_tests;
#[cfg(test)]
mod known_zone_tests;
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
#[cfg(test)]
mod withdraw_error_report_tests;

pub(crate) use change_conflict_next::CHANGE_CONFLICT_NEXT;
pub(crate) use known_zone::known_zone;
pub(crate) use land_error_report::land_error_report;
pub(crate) use landing_state_error_report::landing_state_error_report;
pub(crate) use pick_squash_ref::pick_squash_ref;
pub(crate) use publish_error_report::publish_error_report;
pub(crate) use resolve_squash::resolve_squash;
pub(crate) use squash_error_report::squash_error_report;
pub(crate) use squash_name::SquashName;
pub(crate) use squash_pick::SquashPick;
pub(crate) use squash_pick_report::squash_pick_report;
pub(crate) use withdraw_error_report::withdraw_error_report;

pub(crate) use landing_refusal::landing_refusal;
