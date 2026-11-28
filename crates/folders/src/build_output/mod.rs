//! Sharing a parent folder's build output with a newly made folder through copy-on-write clones.

mod ancestor_link_why;
mod ancestor_refusal;
mod ancestor_walk;
mod build_dir_clone;
mod build_dir_outcome;
mod clone_build_dirs;
#[cfg(target_os = "macos")]
mod dir_fd;
#[cfg(not(target_os = "macos"))]
mod dir_path;
mod entry;
mod walk_ancestors;

#[cfg(test)]
mod ancestor_link_why_tests;
#[cfg(test)]
mod ancestor_refusal_tests;
#[cfg(test)]
mod build_dir_outcome_tests;

pub(crate) use ancestor_link_why::ancestor_link_why;
pub(crate) use ancestor_refusal::ancestor_refusal;
pub(crate) use ancestor_walk::AncestorWalk;
pub use build_dir_clone::BuildDirClone;
pub(crate) use build_dir_outcome::build_dir_outcome;
pub use clone_build_dirs::clone_build_dirs;
#[cfg(target_os = "macos")]
pub(crate) use dir_fd::DirFd;
#[cfg(not(target_os = "macos"))]
pub(crate) use dir_path::DirFd;
pub(crate) use entry::Entry;
pub(crate) use walk_ancestors::walk_ancestors;
