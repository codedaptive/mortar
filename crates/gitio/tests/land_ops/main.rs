//! Real-git joins for the operations land needs: push, fetch_ref, ls_remote, hooks_path, first_parent_list, is_ancestor, worktree_fill.
#[path = "../support/mod.rs"]
mod support;
#[path = "../typed/fixture/mod.rs"]
#[allow(dead_code, unused_imports)]
mod fixture;
mod local_reads;
mod push;
mod remote_reads;
mod worktree_fill;
