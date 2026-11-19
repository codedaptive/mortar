//! The folder verbs through the built mortar and mr binaries, against throwaway repositories with real Git: each
//! run_folder_* module drives one folder verb (zone, request, remove, attempts, keep, drop).
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod run_folder_zone_tests;
mod run_folder_request_tests;
mod run_folder_remove_tests;
mod run_folder_attempts_tests;
mod run_folder_keep_tests;
mod run_folder_drop_tests;
