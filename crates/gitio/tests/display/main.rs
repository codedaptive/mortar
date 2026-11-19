//! GitError, TreeEntryError and TreeWithPathsError Display text, as std::error::Error shows it and as the two
//! From<..> for GitError impls store it, through the public API only. Unit tests (ZUTs): runs no git and touches
//! no files; the functions exercised only format or convert errors, which is pure code of the kit.
mod git_error;
mod git_error_from_tree_entry_error;
mod git_error_from_tree_with_paths_error;
mod tree_entry_error;
mod tree_with_paths_error;
