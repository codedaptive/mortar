//! log, restore and the Git version floor through the built mortar and mr binaries, against throwaway repositories
//! with real Git.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod run_log_tests;
mod run_restore_tests;
mod run_restore_repeat_tests;
mod run_git_floor_tests;
