//! Installed-guard validation through the built CLI binary and real Git on disposable repositories.
//! Fixture and JSON helpers are shared with cli_rit without copying or changing their behavior.

// These shared modules also serve the larger cli_rit suite; validation uses only part of their public helpers.
#[allow(dead_code)]
#[path = "../cli_rit/json.rs"]
mod json;
#[allow(dead_code, unused_imports)]
#[path = "../cli_rit/support.rs"]
mod support;
mod guard_linked_worktree;
mod guard_symref;
