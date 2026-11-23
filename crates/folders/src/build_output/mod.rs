//! Sharing a parent folder's build output with a newly made folder through copy-on-write clones.

mod ancestor_refusal;
mod build_dir_clone;
mod build_dir_outcome;
mod clone_build_dirs;
mod clonefile_dir;
mod symlinked_ancestor;

#[cfg(test)]
mod ancestor_refusal_tests;
#[cfg(test)]
mod build_dir_outcome_tests;

pub(crate) use ancestor_refusal::ancestor_refusal;
pub use build_dir_clone::BuildDirClone;
pub(crate) use build_dir_outcome::build_dir_outcome;
pub use clone_build_dirs::clone_build_dirs;
pub(crate) use clonefile_dir::clonefile_dir;
pub(crate) use symlinked_ancestor::symlinked_ancestor;
