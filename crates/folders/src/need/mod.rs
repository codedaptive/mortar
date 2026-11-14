//! Needing a path outside the work order: adding it to a folder's cone and record, and listing the gaps.

mod add_needed;
mod gaps;
mod need;
mod need_path;

#[cfg(test)]
mod add_needed_tests;
#[cfg(test)]
mod gaps_tests;
#[cfg(test)]
mod need_path_tests;

pub use add_needed::add_needed;
pub use gaps::gaps;
pub use need::need;
pub(crate) use need_path::need_path;
