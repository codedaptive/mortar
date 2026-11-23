//! Sharing a parent folder's build output with a newly made folder through copy-on-write clones.

mod build_dir_clone;
mod build_dir_outcome;
mod clone_build_dirs;
mod clonefile_dir;

#[cfg(test)]
mod build_dir_outcome_tests;

pub use build_dir_clone::BuildDirClone;
pub(crate) use build_dir_outcome::build_dir_outcome;
pub use clone_build_dirs::clone_build_dirs;
pub(crate) use clonefile_dir::clonefile_dir;
