//! The folder an agent verb runs in (S-18), what a failure to read it reports, and who each verb acts as.

mod actor_for;
mod context_error;
mod context_report;
mod folder_context;
mod read_folder_context;
mod repository_error;
mod repository_report;

#[cfg(test)]
mod actor_for_tests;
#[cfg(test)]
mod repository_error_tests;
#[cfg(test)]
mod repository_report_tests;

pub use actor_for::actor_for;
pub use context_error::ContextError;
pub use context_report::context_report;
pub use folder_context::FolderContext;
pub use read_folder_context::read_folder_context;

pub(crate) use repository_error::repository_error;
pub(crate) use repository_report::repository_report;
