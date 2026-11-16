//! The cli's words for land's refusals and failures, shared by the landing verbs: squash's, the landing state
//! file's, land's and back-out's, and publish's.

mod land_error_report;
mod landing_refusal;
mod landing_state_error_report;
mod publish_error_report;
mod squash_error_report;

#[cfg(test)]
mod land_error_report_tests;
#[cfg(test)]
mod landing_refusal_tests;
#[cfg(test)]
mod landing_state_error_report_tests;
#[cfg(test)]
mod publish_error_report_tests;
#[cfg(test)]
mod squash_error_report_tests;

pub use land_error_report::land_error_report;
pub use landing_state_error_report::landing_state_error_report;
pub use publish_error_report::publish_error_report;
pub use squash_error_report::squash_error_report;

pub(crate) use landing_refusal::landing_refusal;
