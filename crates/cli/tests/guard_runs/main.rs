//! The installed guard through the built mortar and mr binaries, against throwaway repositories with real Git: guard
//! drives the installed guard hooks, guard_* cover the guard and its PATH lookup, and run_guard_* drive guard install,
//! status and the hooks. cid, refs and guard_fixture are the guard fixtures.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod guard;
mod guard_path;
mod guard_symref;
mod guard_linked_worktree;
mod guard_install_path_lookup;
mod run_guard_install_tests;
mod run_guard_status_tests;
mod run_guard_hook_tests;
#[allow(dead_code)]
mod cid;
#[allow(dead_code)]
mod refs;
#[allow(dead_code)]
mod guard_fixture;
