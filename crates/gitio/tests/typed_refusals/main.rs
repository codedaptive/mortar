//! WO-gitio-14: each typed refusal of Git::tree_with_paths_typed and Git::tree_entry_typed, reached through the real
//! method, and the GitError text the untyped Git::tree_with_paths and Git::tree_entry give for it (join reads that
//! text). Zone integration tests: every case runs git in a throwaway repository.
#[path = "../support/mod.rs"]
mod support;
mod fixture;
mod tree_entry;
mod tree_with_paths;
