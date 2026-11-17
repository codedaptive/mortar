//! GitError, TreeEntryError and TreeWithPathsError Display text, as std::error::Error shows it and as the two
//! From<..> for GitError impls store it, through the public API only. Zone integration tests: runs no git and
//! touches no files; ZITs because each runs another file's code for real (S-102, rule 11).
mod git_error;
mod git_error_from_tree_entry_error;
mod git_error_from_tree_with_paths_error;
mod tree_entry_error;
mod tree_with_paths_error;
