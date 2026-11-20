//! Git::index_tree, the tree a folder's own index holds, and its typed refusals. Zone integration tests:
//! real git in throwaway repositories made by tests/support's GitRepo; expected values come from raw git or literals.
#[path = "../support/mod.rs"]
mod support;
mod fixture;
mod index_forms;
mod reads;
mod refusals;
mod unmerged;
mod untouched;
mod worktrees;
